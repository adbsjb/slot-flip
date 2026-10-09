//! Flip input probe.
//!
//! The Miyoo Flip's pad enumerates as an Xbox-style device and MinUI reads it
//! through SDL's joystick / game-controller API rather than raw evdev, which is
//! why the first smoke test saw no events on /dev/input/event5. This probe
//! brings up SDL exactly the way the real frontend will and logs every button,
//! axis and hat it sees, with the SDL ids, so the button remap can be written
//! against what actually fires on this hardware.
//!
//! It dlopens libSDL2 directly so nothing new is linked at build time. Runs for
//! $SLOT_INPUT_SECS seconds (default 30) then exits back to MinUI.

use std::ffi::{c_char, c_int, c_void, CStr};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use libloading::{Library, Symbol};

const SDL_INIT_VIDEO: u32 = 0x0000_0020;
const SDL_INIT_JOYSTICK: u32 = 0x0000_0200;
const SDL_INIT_GAMECONTROLLER: u32 = 0x0000_2000;

// SDL event types we care about.
const SDL_KEYDOWN: u32 = 0x300;
const SDL_KEYUP: u32 = 0x301;
const SDL_JOYAXISMOTION: u32 = 0x600;
const SDL_JOYHATMOTION: u32 = 0x602;
const SDL_JOYBUTTONDOWN: u32 = 0x603;
const SDL_JOYBUTTONUP: u32 = 0x604;
const SDL_JOYDEVICEADDED: u32 = 0x605;
const SDL_CONTROLLERAXISMOTION: u32 = 0x650;
const SDL_CONTROLLERBUTTONDOWN: u32 = 0x651;
const SDL_CONTROLLERBUTTONUP: u32 = 0x652;
const SDL_CONTROLLERDEVICEADDED: u32 = 0x653;

// SDL_Event is a 56-byte union. We read the first fields we need by offset so
// we don't have to mirror the whole struct. Layout (little endian):
//   u32 type @0
//   common/timestamp @4
// For key events: SDL_KeyboardEvent has keysym at offset 16:
//   SDL_Keysym { Sint32 scancode @16, Sint32 sym @20, u16 mod @24 }
// For joy button: SDL_JoyButtonEvent { which @8 (SDL_JoystickID i32), u8 button @12, u8 state @13 }
// For joy axis:   SDL_JoyAxisEvent  { which @8, u8 axis @12, Sint16 value @16 }
// For joy hat:    SDL_JoyHatEvent   { which @8, u8 hat @12, u8 value @13 }
// For controller button: SDL_ControllerButtonEvent { which @8 (i32), u8 button @12, u8 state @13 }
// For controller axis:   SDL_ControllerAxisEvent   { which @8, u8 axis @12, Sint16 value @16 }
const EVENT_SIZE: usize = 56;

type InitSubSystem = unsafe extern "C" fn(u32) -> c_int;
type GetError = unsafe extern "C" fn() -> *const c_char;
type PollEvent = unsafe extern "C" fn(*mut u8) -> c_int;
type NumJoysticks = unsafe extern "C" fn() -> c_int;
type IsGameController = unsafe extern "C" fn(c_int) -> c_int;
type JoystickOpen = unsafe extern "C" fn(c_int) -> *mut c_void;
type JoystickName = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type GameControllerOpen = unsafe extern "C" fn(c_int) -> *mut c_void;
type GameControllerNameForIndex = unsafe extern "C" fn(c_int) -> *const c_char;
type ControllerButtonName = unsafe extern "C" fn(c_int) -> *const c_char;
type ControllerAxisName = unsafe extern "C" fn(c_int) -> *const c_char;
type Delay = unsafe extern "C" fn(u32);

fn u32_at(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}
fn i32_at(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}
fn i16_at(buf: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([buf[off], buf[off + 1]])
}

struct Log(Option<std::fs::File>);
impl Log {
    fn open() -> Self {
        let path = std::env::var("SLOT_LOG").unwrap_or_else(|_| "/mnt/sdcard/slot-input.log".into());
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

unsafe fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

fn main() {
    let mut log = Log::open();
    log.line("== flip input probe ==");

    let lib = match unsafe { Library::new("libSDL2-2.0.so.0") }
        .or_else(|_| unsafe { Library::new("libSDL2.so") })
    {
        Ok(l) => l,
        Err(e) => {
            log.line(&format!("libSDL2: {e}"));
            std::process::exit(1);
        }
    };

    macro_rules! get {
        ($t:ty, $name:expr) => {{
            let s: Symbol<$t> = unsafe { lib.get($name.as_bytes()).expect($name) };
            *s
        }};
    }
    let init_subsystem: InitSubSystem = get!(InitSubSystem, "SDL_InitSubSystem");
    let get_error: GetError = get!(GetError, "SDL_GetError");
    let poll_event: PollEvent = get!(PollEvent, "SDL_PollEvent");
    let num_joysticks: NumJoysticks = get!(NumJoysticks, "SDL_NumJoysticks");
    let is_gc: IsGameController = get!(IsGameController, "SDL_IsGameController");
    let joy_open: JoystickOpen = get!(JoystickOpen, "SDL_JoystickOpen");
    let joy_name: JoystickName = get!(JoystickName, "SDL_JoystickName");
    let gc_open: GameControllerOpen = get!(GameControllerOpen, "SDL_GameControllerOpen");
    let gc_name_idx: GameControllerNameForIndex =
        get!(GameControllerNameForIndex, "SDL_GameControllerNameForIndex");
    let btn_name: ControllerButtonName =
        get!(ControllerButtonName, "SDL_GameControllerGetStringForButton");
    let axis_name: ControllerAxisName =
        get!(ControllerAxisName, "SDL_GameControllerGetStringForAxis");
    let delay: Delay = get!(Delay, "SDL_Delay");

    unsafe {
        // Video so the driver/window path matches the real frontend; the pad
        // only reports under joystick + gamecontroller.
        if init_subsystem(SDL_INIT_VIDEO) != 0 {
            log.line(&format!("init video: {}", cstr(get_error())));
        }
        if init_subsystem(SDL_INIT_JOYSTICK | SDL_INIT_GAMECONTROLLER) != 0 {
            log.line(&format!("init joystick/gc: {}", cstr(get_error())));
        }

        let n = num_joysticks();
        log.line(&format!("SDL_NumJoysticks = {n}"));
        for i in 0..n {
            let gc = is_gc(i) != 0;
            log.line(&format!(
                "device {i}: gamecontroller={gc} name={:?}",
                cstr(gc_name_idx(i))
            ));
            if gc {
                let h = gc_open(i);
                log.line(&format!("  SDL_GameControllerOpen({i}) -> {}", !h.is_null()));
            } else {
                let h = joy_open(i);
                log.line(&format!(
                    "  SDL_JoystickOpen({i}) -> {} name={:?}",
                    !h.is_null(),
                    cstr(joy_name(h))
                ));
            }
        }
        log.line("press every button now; axes/hats will log too");
    }

    let secs: u64 = std::env::var("SLOT_INPUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let deadline = Instant::now() + Duration::from_secs(secs);
    let mut ev = [0u8; EVENT_SIZE];

    while Instant::now() < deadline {
        let mut any = false;
        while unsafe { poll_event(ev.as_mut_ptr()) } == 1 {
            any = true;
            let ty = u32_at(&ev, 0);
            match ty {
                SDL_KEYDOWN | SDL_KEYUP => {
                    let scancode = i32_at(&ev, 16);
                    let sym = i32_at(&ev, 20);
                    let dir = if ty == SDL_KEYDOWN { "down" } else { "up" };
                    log.line(&format!("KEY {dir} scancode={scancode} sym={sym}"));
                }
                SDL_JOYBUTTONDOWN | SDL_JOYBUTTONUP => {
                    let which = i32_at(&ev, 8);
                    let button = ev[12];
                    let dir = if ty == SDL_JOYBUTTONDOWN { "down" } else { "up" };
                    log.line(&format!("JOYBUTTON {dir} which={which} button={button}"));
                }
                SDL_JOYHATMOTION => {
                    let which = i32_at(&ev, 8);
                    let hat = ev[12];
                    let value = ev[13];
                    log.line(&format!("JOYHAT which={which} hat={hat} value={value}"));
                }
                SDL_JOYAXISMOTION => {
                    let which = i32_at(&ev, 8);
                    let axis = ev[12];
                    let value = i16_at(&ev, 16);
                    // Ignore near-centre jitter.
                    if value.abs() > 12000 {
                        log.line(&format!("JOYAXIS which={which} axis={axis} value={value}"));
                    }
                }
                SDL_CONTROLLERBUTTONDOWN | SDL_CONTROLLERBUTTONUP => {
                    let which = i32_at(&ev, 8);
                    let button = ev[12] as c_int;
                    let dir = if ty == SDL_CONTROLLERBUTTONDOWN { "down" } else { "up" };
                    let name = unsafe { cstr(btn_name(button)) };
                    log.line(&format!(
                        "CONTROLLERBUTTON {dir} which={which} button={button} ({name})"
                    ));
                }
                SDL_CONTROLLERAXISMOTION => {
                    let which = i32_at(&ev, 8);
                    let axis = ev[12] as c_int;
                    let value = i16_at(&ev, 16);
                    if value.abs() > 12000 {
                        let name = unsafe { cstr(axis_name(axis)) };
                        log.line(&format!(
                            "CONTROLLERAXIS which={which} axis={axis} ({name}) value={value}"
                        ));
                    }
                }
                SDL_JOYDEVICEADDED => log.line("JOYDEVICEADDED"),
                SDL_CONTROLLERDEVICEADDED => log.line("CONTROLLERDEVICEADDED"),
                _ => {}
            }
        }
        if !any {
            unsafe { delay(8) };
        }
    }

    log.line("== done, exiting ==");
}
