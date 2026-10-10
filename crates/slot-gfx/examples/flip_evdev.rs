//! Raw evdev probe for the Flip.
//!
//! The first smoke test read /dev/input/event5 directly and saw nothing, most
//! likely because MinUI's miyoo_inputd/keymon holds an exclusive grab on the
//! pad. This probe is more thorough:
//!   * enumerates every /dev/input/event* node and prints its name + key caps,
//!   * reads ALL of them at once (threads), so a grabbed node simply stays
//!     quiet while the live one reports,
//!   * decodes EV_KEY/EV_ABS/EV_SW with the exact code so we can build the map,
//!   * optionally tries to EVIOCGRAB each node itself if SLOT_GRAB=1 (reports
//!     whether the grab succeeded, which tells us if something else holds it).
//!
//! No SDL, no window; pure evdev. Runs for $SLOT_EVDEV_SECS (default 30s).

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const EVENT_BYTES: usize = 24;
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const EV_SW: u16 = 0x05;

// ioctl request for EVIOCGRAB (write int). _IOW('E', 0x90, int) on arm64.
const EVIOCGRAB: libc_ulong = 0x4004_4590;
#[allow(non_camel_case_types)]
type libc_ulong = std::os::raw::c_ulong;

extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: libc_ulong, ...) -> std::os::raw::c_int;
}

fn decode(b: &[u8]) -> Option<(u16, u16, i32)> {
    let p: &[u8; EVENT_BYTES] = b.get(..EVENT_BYTES)?.try_into().ok()?;
    Some((
        u16::from_le_bytes([p[16], p[17]]),
        u16::from_le_bytes([p[18], p[19]]),
        i32::from_le_bytes([p[20], p[21], p[22], p[23]]),
    ))
}

fn kind_name(k: u16) -> &'static str {
    match k {
        EV_KEY => "KEY",
        EV_ABS => "ABS",
        EV_SW => "SW ",
        _ => "?? ",
    }
}

// Common BTN_ names so the log is readable without a lookup table handy.
fn btn_name(code: u16) -> &'static str {
    match code {
        0x130 => "BTN_A/SOUTH",
        0x131 => "BTN_B/EAST",
        0x132 => "BTN_C",
        0x133 => "BTN_X/NORTH",
        0x134 => "BTN_Y/WEST",
        0x135 => "BTN_Z",
        0x136 => "BTN_TL (L1)",
        0x137 => "BTN_TR (R1)",
        0x138 => "BTN_TL2 (L2)",
        0x139 => "BTN_TR2 (R2)",
        0x13a => "BTN_SELECT",
        0x13b => "BTN_START",
        0x13c => "BTN_MODE (menu)",
        0x13d => "BTN_THUMBL",
        0x13e => "BTN_THUMBR",
        114 => "KEY_VOLUMEDOWN",
        115 => "KEY_VOLUMEUP",
        116 => "KEY_POWER",
        _ => "",
    }
}

struct Log(Option<File>);
impl Log {
    fn open() -> Self {
        let path = std::env::var("SLOT_LOG").unwrap_or_else(|_| "/mnt/sdcard/slot-evdev.log".into());
        Log(OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .ok())
    }
    fn line(&mut self, s: &str) {
        eprintln!("{s}");
        if let Some(f) = self.0.as_mut() {
            let _ = writeln!(f, "{s}");
            let _ = f.flush();
        }
    }
}

fn read_sysfs(p: &Path) -> String {
    std::fs::read_to_string(p)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn nodes() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir("/dev/input")
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("event"))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn main() {
    let mut log = Log::open();
    log.line("== flip evdev probe ==");

    let try_grab = std::env::var("SLOT_GRAB").ok().as_deref() == Some("1");
    let secs: u64 = std::env::var("SLOT_EVDEV_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let found = nodes();
    log.line(&format!("found {} event nodes", found.len()));
    for node in &found {
        let name = node.file_name().unwrap().to_string_lossy().into_owned();
        let sys = Path::new("/sys/class/input").join(&name).join("device");
        let dev_name = read_sysfs(&sys.join("name"));
        let keys = read_sysfs(&sys.join("capabilities/key"));
        log.line(&format!("{} : \"{}\" key={}", node.display(), dev_name, keys));
    }
    log.line("--- press EVERY button now, one at a time, incl R1, L2, R2, volume ---");

    let (tx, rx) = mpsc::channel::<String>();
    for node in found {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let label = node.file_name().unwrap().to_string_lossy().into_owned();
            let file = match File::open(&node) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(format!("{label}: open failed: {e}"));
                    return;
                }
            };
            if try_grab {
                let rc = unsafe { ioctl(file.as_raw_fd(), EVIOCGRAB, 1) };
                let _ = tx.send(format!(
                    "{label}: EVIOCGRAB -> {}",
                    if rc == 0 { "ok (nothing else held it)" } else { "FAILED (held elsewhere)" }
                ));
            }
            let mut file = file;
            let mut buf = [0u8; EVENT_BYTES * 32];
            loop {
                let n = match file.read(&mut buf) {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        let _ = tx.send(format!("{label}: read failed: {e}"));
                        return;
                    }
                };
                for ev in buf[..n].chunks_exact(EVENT_BYTES).filter_map(decode) {
                    let (kind, code, value) = ev;
                    if kind == EV_SYN {
                        continue;
                    }
                    if (kind == EV_KEY || kind == EV_SW) || (kind == EV_ABS && value.abs() > 8000) {
                        let name = btn_name(code);
                        let _ = tx.send(format!(
                            "{label} {} code={} (0x{:x}) value={} {}",
                            kind_name(kind),
                            code,
                            code,
                            value,
                            name
                        ));
                    }
                }
            }
        });
    }
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(msg) => log.line(&msg),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    log.line("== done, exiting ==");
}
