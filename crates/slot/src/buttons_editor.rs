//! State behind the Buttons screen: which row is picked, whether it is waiting
//! for a key, and what to tell the player afterwards. It owns no drawing and no
//! input device; it talks to the live map in [`crate::input::keymap`], which is
//! where the physical key actually gets captured.

use slot_input::{Action, Btn};
use slot_ui::{ButtonsRow, ButtonsView};

use crate::input::keymap::{self, Outcome};

/// The rows, in reading order: first column, then second. `None` is the
/// "Reset" action row.
const ROWS: [Option<Btn>; 16] = [
    Some(Btn::Up),
    Some(Btn::Down),
    Some(Btn::Left),
    Some(Btn::Right),
    Some(Btn::A),
    Some(Btn::B),
    Some(Btn::X),
    Some(Btn::Y),
    Some(Btn::L1),
    Some(Btn::R1),
    Some(Btn::L2),
    Some(Btn::R2),
    Some(Btn::Start),
    Some(Btn::Select),
    Some(Btn::Menu),
    None,
];

/// Rows in the first column; Left and Right jump by this much.
const COLUMN: usize = ROWS.len() / 2;

/// How long the screen waits for a key before giving up.
pub const CAPTURE_MS: f64 = 6000.0;

#[derive(Default)]
pub struct ButtonsEditor {
    cursor: usize,
    /// The button being rebound and when the wait ends (clock milliseconds).
    capture: Option<(Btn, f64)>,
    notice: String,
    shown_secs: u32,
    rev: u32,
}

fn label(btn: Btn) -> String {
    match btn {
        Btn::Up => "Up",
        Btn::Down => "Down",
        Btn::Left => "Left",
        Btn::Right => "Right",
        Btn::A => "A",
        Btn::B => "B",
        Btn::X => "X",
        Btn::Y => "Y",
        Btn::L1 => "L1",
        Btn::R1 => "R1",
        Btn::L2 => "L2",
        Btn::R2 => "R2",
        Btn::Start => "Start",
        Btn::Select => "Select",
        Btn::Menu => "Menu",
        other => other.as_name(),
    }
    .to_string()
}

fn key_label(btn: Btn) -> String {
    let keys = keymap::keys_for(btn);
    if keys.is_empty() {
        return "none".to_string();
    }
    keys.iter()
        .map(|&c| keymap::key_name(c))
        .collect::<Vec<_>>()
        .join(" / ")
}

impl ButtonsEditor {
    /// Changes whenever the picture would change, so the face is only rebuilt
    /// when it has to be.
    pub fn rev(&self) -> u32 {
        self.rev
    }

    pub fn capturing(&self) -> bool {
        self.capture.is_some()
    }

    fn bump(&mut self) {
        self.rev = self.rev.wrapping_add(1);
    }

    /// Start from a clean screen each time it is opened.
    pub fn enter(&mut self) {
        self.cancel();
        self.cursor = 0;
        self.notice.clear();
        self.bump();
    }

    /// Stop waiting for a key, if it was.
    pub fn cancel(&mut self) {
        if self.capture.take().is_some() {
            keymap::capture_cancel();
            self.bump();
        }
    }

    /// Handle a menu action. Returns true when the player asked to leave.
    pub fn input(&mut self, action: Action, now: f64) -> bool {
        if self.capture.is_some() {
            // Keys are being captured; nothing else should act meanwhile.
            return false;
        }
        let last = ROWS.len() - 1;
        let before = self.cursor;
        match action {
            Action::GbaDown(Btn::Up) => self.cursor = self.cursor.saturating_sub(1),
            Action::GbaDown(Btn::Down) => self.cursor = (self.cursor + 1).min(last),
            Action::GbaDown(Btn::Left) => self.cursor = self.cursor.saturating_sub(COLUMN),
            Action::GbaDown(Btn::Right) => self.cursor = (self.cursor + COLUMN).min(last),
            Action::GbaDown(Btn::A) => self.activate(now),
            Action::GbaDown(Btn::B) | Action::QuickMenu => return true,
            _ => {}
        }
        if self.cursor != before {
            self.bump();
        }
        false
    }

    fn activate(&mut self, now: f64) {
        match ROWS[self.cursor] {
            Some(btn) => {
                keymap::capture_arm();
                self.capture = Some((btn, now + CAPTURE_MS));
                self.shown_secs = 0;
                self.notice.clear();
            }
            None => {
                self.notice = match keymap::reset() {
                    Ok(()) => "Built-in buttons restored".to_string(),
                    Err(e) => {
                        eprintln!("slot: buttons.ini: {e}");
                        "Built-in buttons restored (could not save)".to_string()
                    }
                };
            }
        }
        self.bump();
    }

    /// Poll for the captured key and the timeout. Call every frame.
    pub fn tick(&mut self, now: f64) {
        let Some((btn, deadline)) = self.capture else {
            return;
        };
        if let Some(code) = keymap::capture_take() {
            self.capture = None;
            self.bind(btn, code);
            self.bump();
        } else if now >= deadline {
            keymap::capture_cancel();
            self.capture = None;
            self.notice = "No key pressed, nothing changed".to_string();
            self.bump();
        } else {
            let secs = ((deadline - now) / 1000.0).ceil() as u32;
            if secs != self.shown_secs {
                self.shown_secs = secs;
                self.bump();
            }
        }
    }

    fn bind(&mut self, btn: Btn, code: u16) {
        let (outcome, saved) = keymap::rebind(btn, code);
        let key = keymap::key_name(code);
        self.notice = match outcome {
            Outcome::Already => format!("{} is already {key}", label(btn)),
            Outcome::Bound { swapped: None } => format!("{} is now {key}", label(btn)),
            Outcome::Bound {
                swapped: Some(prev),
            } => format!(
                "{} is now {key}; {} moved to {}",
                label(btn),
                label(prev),
                key_label(prev)
            ),
        };
        if let Err(e) = saved {
            eprintln!("slot: buttons.ini: {e}");
            self.notice.push_str(" (not saved)");
        }
    }

    pub fn view(&self) -> ButtonsView {
        let rows = ROWS
            .iter()
            .map(|row| match row {
                Some(btn) => ButtonsRow {
                    label: label(*btn),
                    keys: key_label(*btn),
                },
                None => ButtonsRow {
                    label: "Reset to built-in".to_string(),
                    keys: String::new(),
                },
            })
            .collect();
        ButtonsView {
            rows,
            cursor: self.cursor,
            prompt: self
                .capture
                .map(|(btn, _)| (label(btn), self.shown_secs.max(1))),
            notice: self.notice.clone(),
        }
    }
}
