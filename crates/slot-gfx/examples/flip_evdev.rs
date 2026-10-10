//! Raw evdev probe for the Flip (thorough edition).
//!
//! Reads EVERY /dev/input/event* node at once and records every EV_KEY / EV_SW
//! press and the full range of every EV_ABS axis. Nothing is thresholded or
//! filtered, so even a light analogue trigger press is captured.
//!
//! Instead of a fixed timer it runs until you press Volume-Up and Volume-Down
//! together, then writes a clean SUMMARY: every distinct key code seen on each
//! node, and the min/max of every axis. That summary is all we need to build
//! the button map with certainty, no matter what order things were pressed.
//!
//! Output: $SLOT_LOG (default /mnt/sdcard/slot-evdev.log). A full event trace
//! goes to $SLOT_LOG.trace for reference.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const EVENT_BYTES: usize = 24;
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const EV_SW: u16 = 0x05;

const KEY_VOLUMEUP: u16 = 115;
const KEY_VOLUMEDOWN: u16 = 114;

fn decode(b: &[u8]) -> Option<(u16, u16, i32)> {
    let p: &[u8; EVENT_BYTES] = b.get(..EVENT_BYTES)?.try_into().ok()?;
    Some((
        u16::from_le_bytes([p[16], p[17]]),
        u16::from_le_bytes([p[18], p[19]]),
        i32::from_le_bytes([p[20], p[21], p[22], p[23]]),
    ))
}

fn key_name(code: u16) -> &'static str {
    match code {
        // event5 gamepad BTN_*
        0x130 => "BTN_A/SOUTH",
        0x131 => "BTN_B/EAST",
        0x133 => "BTN_X/NORTH",
        0x134 => "BTN_Y/WEST",
        0x136 => "BTN_TL (L1)",
        0x137 => "BTN_TR (R1)",
        0x138 => "BTN_TL2 (L2)",
        0x139 => "BTN_TR2 (R2)",
        0x13a => "BTN_SELECT",
        0x13b => "BTN_START",
        0x13c => "BTN_MODE",
        // event0 gpio keyboard codes
        103 => "KEY_UP",
        108 => "KEY_DOWN",
        105 => "KEY_LEFT",
        106 => "KEY_RIGHT",
        57 => "KEY_SPACE",
        29 => "KEY_LEFTCTRL",
        42 => "KEY_LEFTSHIFT",
        56 => "KEY_LEFTALT",
        15 => "KEY_TAB",
        104 => "KEY_PAGEUP",
        109 => "KEY_PAGEDOWN",
        28 => "KEY_ENTER",
        97 => "KEY_RIGHTCTRL",
        1 => "KEY_ESC",
        115 => "KEY_VOLUMEUP",
        114 => "KEY_VOLUMEDOWN",
        116 => "KEY_POWER",
        _ => "",
    }
}

struct Log {
    out: Option<File>,
    trace: Option<File>,
}
impl Log {
    fn open() -> Self {
        let path = std::env::var("SLOT_LOG").unwrap_or_else(|_| "/mnt/sdcard/slot-evdev.log".into());
        let trace_path = format!("{path}.trace");
        let mk = |p: &str| {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(p)
                .ok()
        };
        Log {
            out: mk(&path),
            trace: mk(&trace_path),
        }
    }
    fn line(&mut self, s: &str) {
        eprintln!("{s}");
        if let Some(f) = self.out.as_mut() {
            let _ = writeln!(f, "{s}");
            let _ = f.flush();
        }
    }
    fn trace(&mut self, s: &str) {
        if let Some(f) = self.trace.as_mut() {
            let _ = writeln!(f, "{s}");
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

// One record sent from a reader thread.
struct Rec {
    node: String,
    kind: u16,
    code: u16,
    value: i32,
}

fn main() {
    let mut log = Log::open();
    log.line("== flip evdev probe (thorough) ==");

    let found = nodes();
    log.line(&format!("found {} event nodes", found.len()));
    for node in &found {
        let name = node.file_name().unwrap().to_string_lossy().into_owned();
        let sys = Path::new("/sys/class/input").join(&name).join("device");
        log.line(&format!(
            "{} : \"{}\"",
            node.display(),
            read_sysfs(&sys.join("name"))
        ));
    }
    log.line("");
    log.line("Press buttons in any order, as many times as you like.");
    log.line("When finished, hold VOLUME-UP and VOLUME-DOWN together to exit.");
    log.line("");

    let (tx, rx) = mpsc::channel::<Rec>();
    for node in found {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let label = node.file_name().unwrap().to_string_lossy().into_owned();
            let mut file = match File::open(&node) {
                Ok(f) => f,
                Err(_) => return,
            };
            let mut buf = [0u8; EVENT_BYTES * 32];
            loop {
                let n = match file.read(&mut buf) {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => return,
                };
                for (kind, code, value) in buf[..n].chunks_exact(EVENT_BYTES).filter_map(decode) {
                    if kind == EV_SYN {
                        continue;
                    }
                    let _ = tx.send(Rec {
                        node: label.clone(),
                        kind,
                        code,
                        value,
                    });
                }
            }
        });
    }
    drop(tx);

    // Distinct key codes per node: node -> set of codes.
    let mut keys: BTreeMap<String, BTreeMap<u16, u32>> = BTreeMap::new();
    // Axis ranges per node: (node,code) -> (min,max,count).
    let mut axes: BTreeMap<(String, u16), (i32, i32, u32)> = BTreeMap::new();

    let mut vol_up_down = false;
    let mut vol_dn_down = false;
    let mut last_summary = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(r) => {
                log.trace(&format!(
                    "{} kind={} code={} (0x{:x}) value={} {}",
                    r.node,
                    r.kind,
                    r.code,
                    r.code,
                    r.value,
                    key_name(r.code)
                ));
                match r.kind {
                    EV_KEY | EV_SW => {
                        if r.value == 1 {
                            *keys.entry(r.node.clone()).or_default().entry(r.code).or_insert(0) += 1;
                        }
                        // Track the quit combo on whichever node carries volume.
                        if r.code == KEY_VOLUMEUP {
                            vol_up_down = r.value != 0;
                        }
                        if r.code == KEY_VOLUMEDOWN {
                            vol_dn_down = r.value != 0;
                        }
                    }
                    EV_ABS => {
                        let e = axes
                            .entry((r.node.clone(), r.code))
                            .or_insert((r.value, r.value, 0));
                        e.0 = e.0.min(r.value);
                        e.1 = e.1.max(r.value);
                        e.2 += 1;
                    }
                    _ => {}
                }
                if vol_up_down && vol_dn_down {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // Periodic progress so the user can see presses are landing.
        if last_summary.elapsed() >= Duration::from_secs(2) {
            last_summary = Instant::now();
            let total: usize = keys.values().map(|m| m.len()).sum();
            eprintln!("...{total} distinct keys seen so far, {} axes", axes.len());
        }
    }

    log.line("================ SUMMARY ================");
    for (node, codes) in &keys {
        log.line(&format!("[{node}] keys:"));
        for (code, count) in codes {
            log.line(&format!(
                "    code {:>4} (0x{:03x})  x{:<3}  {}",
                code,
                code,
                count,
                key_name(*code)
            ));
        }
    }
    if !axes.is_empty() {
        log.line("axes (min .. max):");
        for ((node, code), (lo, hi, cnt)) in &axes {
            log.line(&format!(
                "    [{node}] axis {:>2}  {:>7} .. {:<7}  (n={})",
                code, lo, hi, cnt
            ));
        }
    }
    log.line("========================================");
    log.line("done, exiting");
}
