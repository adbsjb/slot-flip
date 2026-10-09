//! Flip smoke test.
//!
//! Proves the SDL2 + GLES2 surface works on the Miyoo Flip (my355) and captures
//! the gamepad's raw evdev button codes so the real input remap can be written.
//!
//! It opens the SDL surface, clears the screen to a slowly cycling colour so a
//! living display is obvious, and in parallel reads /dev/input/event5 ("MIYOO
//! Player1"), logging every key/abs event. Everything is written both to stderr
//! and to $SLOT_LOG (default /mnt/sdcard/slot-smoke.log) so it can be read off
//! the card afterwards. It runs for $SLOT_SMOKE_SECS seconds (default 20) then
//! exits cleanly back to MinUI.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use slot_gfx::{SdlSurface, Surface};

const EVENT_BYTES: usize = 24;
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const EV_SW: u16 = 0x05;

fn decode(b: &[u8]) -> Option<(u16, u16, i32)> {
    let p: &[u8; EVENT_BYTES] = b.get(..EVENT_BYTES)?.try_into().ok()?;
    Some((
        u16::from_le_bytes([p[16], p[17]]),
        u16::from_le_bytes([p[18], p[19]]),
        i32::from_le_bytes([p[20], p[21], p[22], p[23]]),
    ))
}

fn kind_name(kind: u16) -> &'static str {
    match kind {
        EV_KEY => "KEY",
        EV_ABS => "ABS",
        EV_SW => "SW ",
        _ => "???",
    }
}

struct Logger {
    file: Option<File>,
}

impl Logger {
    fn open() -> Self {
        let path = std::env::var("SLOT_LOG").unwrap_or_else(|_| "/mnt/sdcard/slot-smoke.log".into());
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .ok();
        Logger { file }
    }

    fn line(&mut self, s: &str) {
        eprintln!("{s}");
        if let Some(f) = self.file.as_mut() {
            let _ = writeln!(f, "{s}");
            let _ = f.flush();
        }
    }
}

fn spawn_input_reader(node: &str, tx: mpsc::Sender<String>) {
    let node = node.to_string();
    std::thread::spawn(move || {
        let mut file = match File::open(&node) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(format!("input {node}: open failed: {e}"));
                return;
            }
        };
        let _ = tx.send(format!("input {node}: open ok, reading events"));
        let mut buf = [0u8; EVENT_BYTES * 32];
        loop {
            let n = match file.read(&mut buf) {
                Ok(0) => return,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    let _ = tx.send(format!("input {node}: read failed: {e}"));
                    return;
                }
            };
            for ev in buf[..n].chunks_exact(EVENT_BYTES).filter_map(decode) {
                let (kind, code, value) = ev;
                if kind == EV_SYN {
                    continue;
                }
                // Only report presses/edges, not the flood of zero releases.
                if (kind == EV_KEY || kind == EV_SW) || (kind == EV_ABS && value != 0) {
                    let _ = tx.send(format!(
                        "EVENT {} code={} (0x{:x}) value={}",
                        kind_name(kind),
                        code,
                        code,
                        value
                    ));
                }
            }
        }
    });
}

fn main() {
    let mut log = Logger::open();
    log.line("== flip smoke test ==");

    // Which input node to watch. event5 is the gamepad on my355; allow override.
    let node = std::env::var("SLOT_INPUT").unwrap_or_else(|_| "/dev/input/event5".into());
    let secs: u64 = std::env::var("SLOT_SMOKE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let mut surface = match SdlSurface::new() {
        Ok(s) => s,
        Err(e) => {
            log.line(&format!("SdlSurface::new failed: {e}"));
            std::process::exit(1);
        }
    };
    slot_gfx::load_gl(&surface);
    let (w, h) = surface.window_size();
    log.line(&format!("surface ok, drawable {w}x{h}"));
    unsafe {
        let ver = gl::GetString(gl::VERSION);
        let ren = gl::GetString(gl::RENDERER);
        if !ver.is_null() {
            log.line(&format!(
                "GL_VERSION = {}",
                std::ffi::CStr::from_ptr(ver as *const _).to_string_lossy()
            ));
        }
        if !ren.is_null() {
            log.line(&format!(
                "GL_RENDERER = {}",
                std::ffi::CStr::from_ptr(ren as *const _).to_string_lossy()
            ));
        }
    }

    if Path::new(&node).exists() {
        let (tx, rx) = mpsc::channel();
        spawn_input_reader(&node, tx);
        run_loop(&mut surface, &mut log, secs, Some(rx));
    } else {
        log.line(&format!("input node {node} not present, drawing only"));
        run_loop(&mut surface, &mut log, secs, None);
    }

    log.line("== done, exiting ==");
}

fn run_loop(
    surface: &mut SdlSurface,
    log: &mut Logger,
    secs: u64,
    rx: Option<mpsc::Receiver<String>>,
) {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(secs);
    let mut frame: u64 = 0;
    while Instant::now() < deadline {
        let t = start.elapsed().as_secs_f32();
        // Slow colour cycle so a live panel is unmistakable.
        let r = 0.5 + 0.5 * (t * 0.9).sin();
        let g = 0.5 + 0.5 * (t * 0.7 + 2.0).sin();
        let b = 0.5 + 0.5 * (t * 0.5 + 4.0).sin();
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        if let Err(e) = surface.swap() {
            log.line(&format!("swap failed: {e}"));
            return;
        }
        if let Some(rx) = rx.as_ref() {
            while let Ok(msg) = rx.try_recv() {
                log.line(&msg);
            }
        }
        frame += 1;
        std::thread::sleep(Duration::from_millis(16));
    }
    log.line(&format!("drew {frame} frames in {secs}s"));
}
