use std::path::{Path, PathBuf};

use slot_input::{Btn, RawEvent};

pub const EVENT_BYTES: usize = 24;

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_SW: u16 = 0x05;
pub const SW_LID: u16 = 0x00;

pub const EV_ABS: u16 = 0x03;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;

// Right analogue stick, used on the Flip to drive fast-forward since the
// physical R2 shoulder is repurposed to stand in for a dead R1.
#[cfg(feature = "flip")]
pub const ABS_RX: u16 = 0x03;
#[cfg(feature = "flip")]
const STICK_ON: i32 = 20_000;
#[cfg(feature = "flip")]
const STICK_OFF: i32 = 12_000;

#[derive(Default)]
pub struct Hat {
    x: i32,
    y: i32,
}

impl Hat {
    pub fn feed(&mut self, ev: Ev) -> Vec<RawEvent> {
        let (held, ends) = match ev.code {
            ABS_HAT0X => (&mut self.x, [Btn::Left, Btn::Right]),
            ABS_HAT0Y => (&mut self.y, [Btn::Up, Btn::Down]),
            _ => return Vec::new(),
        };
        let (was, now) = (*held, ev.value.signum());
        if now == was {
            return Vec::new();
        }
        *held = now;
        let btn = |v: i32| ends[usize::from(v > 0)];
        let mut out = Vec::new();
        if was != 0 {
            out.push(RawEvent::Up(btn(was)));
        }
        if now != 0 {
            out.push(RawEvent::Down(btn(now)));
        }
        out
    }
}

// Turns the right stick's horizontal axis into R2 presses (fast-forward) with a
// little hysteresis so a resting stick doesn't chatter. Pushing right past
// STICK_ON presses R2; releasing below STICK_OFF lets it go. Left is ignored.
#[cfg(feature = "flip")]
#[derive(Default)]
pub struct Stick {
    held: bool,
}

#[cfg(feature = "flip")]
impl Stick {
    pub fn feed(&mut self, ev: Ev) -> Vec<RawEvent> {
        if ev.code != ABS_RX {
            return Vec::new();
        }
        if !self.held && ev.value >= STICK_ON {
            self.held = true;
            return vec![RawEvent::Down(Btn::R2)];
        }
        if self.held && ev.value < STICK_OFF {
            self.held = false;
            return vec![RawEvent::Up(Btn::R2)];
        }
        Vec::new()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ev {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

pub fn decode(bytes: &[u8]) -> Option<Ev> {
    let packet: &[u8; EVENT_BYTES] = bytes.get(..EVENT_BYTES)?.try_into().ok()?;
    Some(Ev {
        kind: u16::from_le_bytes([packet[16], packet[17]]),
        code: u16::from_le_bytes([packet[18], packet[19]]),
        value: i32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]),
    })
}

pub fn to_raw(ev: Ev) -> Option<RawEvent> {
    let btn = match ev.kind {
        EV_KEY => code_to_btn(ev.code)?,
        EV_SW if ev.code == SW_LID => Btn::Lid,
        _ => return None,
    };
    match ev.value {
        0 => Some(RawEvent::Up(btn)),
        1 => Some(RawEvent::Down(btn)),
        _ => None,
    }
}

// The Anbernic RG SP (H700) pad speaks BTN_* gamepad codes.
#[cfg(not(feature = "flip"))]
pub fn code_to_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x130 => Btn::A,
        0x131 => Btn::B,
        0x132 => Btn::Y,
        0x133 => Btn::X,
        0x134 => Btn::L1,
        0x135 => Btn::R1,
        0x136 => Btn::Select,
        0x137 => Btn::Start,
        0x138 => Btn::Menu,
        0x13a => Btn::L2,
        0x13b => Btn::R2,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

#[cfg(not(feature = "flip"))]
const WANTED: [u16; 15] = [
    0x130, 0x131, 0x132, 0x133, 0x134, 0x135, 0x136, 0x137, 0x138, 0x13a, 0x13b, 0x162, 115, 114,
    116,
];

// The Miyoo Flip (my355) routes every working button through the gpio
// "keyboard" node (event0) as standard Linux key codes. The gamepad node
// (event5) is missing R1 entirely and reports L2/R2 only as analogue axes, so
// event0 is the reliable source. R1 is physically dead on this unit, so it is
// deliberately left unmapped; its actions are recovered from R2 higher up.
#[cfg(feature = "flip")]
pub fn code_to_btn(code: u16) -> Option<Btn> {
    Some(match code {
        103 => Btn::Up,
        108 => Btn::Down,
        105 => Btn::Left,
        106 => Btn::Right,
        29 => Btn::A,          // KEY_LEFTCTRL   (right face)
        57 => Btn::B,          // KEY_SPACE      (bottom face)
        56 => Btn::X,          // KEY_LEFTALT    (top face)
        42 => Btn::Y,          // KEY_LEFTSHIFT  (left face)
        15 => Btn::L1,         // KEY_TAB
        104 => Btn::L2,        // KEY_PAGEUP  (rewind)
        // R1 is physically dead on this unit; the working R2 shoulder takes its
        // place so platform-switch, aspect-reset and Select+R save state all
        // keep working. Fast-forward (normally R2) moves to the right stick.
        109 => Btn::R1,        // KEY_PAGEDOWN (acts as R1)
        97 => Btn::Select,     // KEY_RIGHTCTRL
        28 => Btn::Start,      // KEY_ENTER
        1 => Btn::Menu,        // KEY_ESC
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

#[cfg(feature = "flip")]
const WANTED: [u16; 16] = [
    103, 108, 105, 106, 29, 57, 56, 42, 15, 104, 109, 97, 28, 1, 115, 114,
];

pub use slot_power::has_bit;

pub fn pick_devices(dev: &Path, sys: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dev) else {
        return Vec::new();
    };
    let mut nodes: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("event"))
        })
        .collect();
    nodes.sort();
    nodes.retain(|node| {
        let Some(name) = node.file_name() else {
            return false;
        };
        wanted_node(&sys.join(name).join("device"))
    });
    nodes
}

fn wanted_node(device: &Path) -> bool {
    let cap = |file: &str| std::fs::read_to_string(device.join("capabilities").join(file));
    let keys = cap("key").unwrap_or_default();
    let lid = cap("sw").is_ok_and(|sw| has_bit(&sw, SW_LID));
    lid || WANTED.iter().any(|bit| has_bit(&keys, *bit))
}

pub fn device_name(sys: &Path, node: &Path) -> String {
    node.file_name()
        .and_then(|n| std::fs::read_to_string(sys.join(n).join("device/name")).ok())
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| node.display().to_string())
}
