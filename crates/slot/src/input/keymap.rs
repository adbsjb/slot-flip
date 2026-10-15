//! User-overridable button map.
//!
//! The compiled defaults in [`super::evdev::code_to_btn`] cover the devices we
//! ship for, but any MinUI handheld routes its buttons through evdev with its
//! own kernel key codes. `Slot/Config/buttons.ini` lets a user remap without a
//! rebuild: each line is `<kernel-code> = <button-name>`, e.g.
//!
//! ```ini
//! # Right shoulder sends KEY_PAGEDOWN (109); bind it to R1.
//! 109 = r1
//! 104 = l2
//! ```
//!
//! Codes may be decimal (`109`) or hex (`0x135`). Button names are the ones
//! from [`Btn::from_name`] (a, b, x, y, up, down, left, right, l1, r1, l2, r2,
//! start, select, menu, volup, voldown, power, lid). Any code not listed keeps
//! its built-in mapping, so a partial file only overrides what it names.

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use slot_input::Btn;

pub const BUTTONS_FILE: &str = "Config/buttons.ini";

static OVERRIDES: OnceLock<HashMap<u16, Btn>> = OnceLock::new();

/// Load `Config/buttons.ini` once. Call early in startup, before input opens.
/// Safe to call more than once; only the first call takes effect.
pub fn load(root: &Path) {
    let _ = OVERRIDES.set(parse(&read_text(root)));
}

fn read_text(root: &Path) -> String {
    std::fs::read_to_string(root.join(BUTTONS_FILE)).unwrap_or_default()
}

/// Look up a user override for a kernel key code, if `load` ran and the code is
/// bound in the file.
pub fn override_for(code: u16) -> Option<Btn> {
    OVERRIDES.get().and_then(|m| m.get(&code).copied())
}

/// Current bindings (code -> button) for display in a menu.
pub fn bindings() -> Vec<(u16, Btn)> {
    let mut v: Vec<(u16, Btn)> = OVERRIDES
        .get()
        .map(|m| m.iter().map(|(&c, &b)| (c, b)).collect())
        .unwrap_or_default();
    v.sort_by_key(|&(c, _)| c);
    v
}

fn parse(text: &str) -> HashMap<u16, Btn> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[')
        {
            continue;
        }
        let Some((code, name)) = line.split_once('=') else {
            continue;
        };
        let Some(code) = parse_code(code.trim()) else {
            continue;
        };
        let Some(btn) = Btn::from_name(name.trim()) else {
            continue;
        };
        out.insert(code, btn);
    }
    out
}

fn parse_code(s: &str) -> Option<u16> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u16>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_decimal_and_hex() {
        let m = parse("109 = r1\n0x135 = l1\n# comment\nbad line\n57=a\n");
        assert_eq!(m.get(&109), Some(&Btn::R1));
        assert_eq!(m.get(&0x135), Some(&Btn::L1));
        assert_eq!(m.get(&57), Some(&Btn::A));
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn ignores_unknown_names() {
        let m = parse("1 = nope\n2 = b\n");
        assert_eq!(m.get(&2), Some(&Btn::B));
        assert_eq!(m.len(), 1);
    }

    // OVERRIDES is a process-global OnceLock, so the whole load -> lookup ->
    // to_raw chain is exercised in one test to keep it deterministic.
    #[test]
    fn load_flows_into_override_and_wins_in_to_raw() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("Config")).unwrap();
        // Rebind KEY_SPACE (57), which the Flip default maps to A, over to B.
        std::fs::write(dir.path().join(BUTTONS_FILE), "57 = b\n").unwrap();
        load(dir.path());

        assert_eq!(override_for(57), Some(Btn::B));
        assert_eq!(override_for(999), None, "unlisted code has no override");
        assert_eq!(bindings(), vec![(57u16, Btn::B)]);

        // On the device path, the override must beat the built-in default.
        #[cfg(feature = "flip")]
        {
            use super::super::evdev::{to_raw, Ev, EV_KEY};
            use slot_input::RawEvent;
            let press = Ev {
                kind: EV_KEY,
                code: 57,
                value: 1,
            };
            assert_eq!(to_raw(press), Some(RawEvent::Down(Btn::B)));
        }
    }
}
