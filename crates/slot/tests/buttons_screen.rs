mod common;

use std::sync::Mutex;

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use slot::app::{App, Phase};
use slot::buttons_editor::CAPTURE_MS;
use slot::input::keymap;
use slot_input::{Action, Btn};
use slot_store::{write_slot_state, SlotState};
use slot_ui::QuickRow;
use tempfile::TempDir;

// The key map and the key capture are process-wide, so these tests take turns.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const A_ROW: usize = 4;
const RESET_ROW: usize = 15;

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

/// On the Buttons screen with a clean map, the first row selected.
fn on_buttons_screen() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..SlotState::default()
        },
    )
    .expect("write slot.state");
    let (mut a, _clock) = app_booting_at(d.path(), CLOCK_IS_SET);
    assert!(
        matches!(a.phase(), Phase::Shelf),
        "not on the shelf after boot: {:?}",
        a.phase()
    );
    keymap::load(d.path());
    keymap::reset().expect("reset");
    keymap::capture_cancel();

    a.apply(Action::QuickMenu);
    for _ in 0..QuickRow::Buttons.position() {
        press(&mut a, Btn::Down);
    }
    assert_eq!(a.quick_menu(), Some(QuickRow::Buttons));
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Buttons), "{:?}", a.phase());
    (d, a)
}

fn down_to(a: &mut App, row: usize) {
    for _ in 0..row {
        press(a, Btn::Down);
    }
    assert_eq!(a.buttons_view().cursor, row);
}

/// A key code nothing is bound to.
fn spare_key() -> u16 {
    (0x2f0..0x300)
        .find(|&c| keymap::resolve(c).is_none())
        .expect("an unused key code")
}

#[test]
fn the_screen_lists_every_button_with_its_key() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, a) = on_buttons_screen();
    let view = a.buttons_view();
    let labels: Vec<&str> = view.rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(
        &labels[..8],
        ["Up", "Down", "Left", "Right", "A", "B", "X", "Y"]
    );
    assert_eq!(
        &labels[8..15],
        ["L1", "R1", "L2", "R2", "Start", "Select", "Menu"]
    );
    assert_eq!(view.rows.len(), 16, "fifteen buttons and the reset row");
    let a_row = &view.rows[A_ROW];
    assert!(
        !a_row.keys.is_empty() && a_row.keys != "none",
        "A should show its built-in key, got {:?}",
        a_row.keys
    );
    assert!(view.prompt.is_none());
}

#[test]
fn pressing_a_key_after_choosing_a_button_rebinds_and_saves_it() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (d, mut a) = on_buttons_screen();
    let spare = spare_key();
    let rev_before = a.buttons_rev();

    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);
    let view = a.buttons_view();
    assert_eq!(view.prompt.as_ref().map(|p| p.0.as_str()), Some("A"));
    assert_ne!(a.buttons_rev(), rev_before, "the picture has to be rebuilt");

    // The reader thread would do this when the key goes down.
    assert!(
        keymap::capture_offer(spare, 1),
        "the press was not captured"
    );
    a.tick_ms(a.now() + 1);

    let view = a.buttons_view();
    assert!(view.prompt.is_none(), "still waiting after the key arrived");
    assert!(view.notice.starts_with("A is now"), "{:?}", view.notice);
    assert_eq!(keymap::keys_for(Btn::A), vec![spare]);
    assert_eq!(view.rows[A_ROW].keys, keymap::key_name(spare));

    let file = std::fs::read_to_string(d.path().join("Config/buttons.ini")).expect("buttons.ini");
    assert!(file.contains(&format!("{spare} = a")), "{file}");

    keymap::reset().expect("reset");
}

#[test]
fn taking_a_key_that_another_button_has_swaps_the_two() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    let b_key = keymap::keys_for(Btn::B);
    let a_key = keymap::keys_for(Btn::A);
    assert_eq!((b_key.len(), a_key.len()), (1, 1), "defaults changed");

    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);
    assert!(keymap::capture_offer(b_key[0], 1));
    a.tick_ms(a.now() + 1);

    assert_eq!(keymap::keys_for(Btn::A), b_key);
    assert_eq!(
        keymap::keys_for(Btn::B),
        a_key,
        "B must not be left keyless"
    );
    let notice = a.buttons_view().notice;
    assert!(notice.contains("B moved to"), "{notice:?}");

    keymap::reset().expect("reset");
}

#[test]
fn no_key_within_the_wait_cancels_and_changes_nothing() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    let before = keymap::keys_for(Btn::A);

    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);
    assert!(a.buttons_view().prompt.is_some());

    a.tick_ms(a.now() + CAPTURE_MS as u64 + 1);
    let view = a.buttons_view();
    assert!(view.prompt.is_none());
    assert!(view.notice.contains("nothing changed"), "{:?}", view.notice);
    assert_eq!(keymap::keys_for(Btn::A), before);
    assert!(
        !keymap::capture_offer(spare_key(), 1),
        "the key capture was left armed after the timeout"
    );
}

#[test]
fn the_prompt_counts_down() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);
    let start = a.now();

    a.tick_ms(start + 100);
    let first = a.buttons_view().prompt.expect("prompt").1;
    let rev = a.buttons_rev();
    a.tick_ms(start + 150);
    assert_eq!(
        a.buttons_rev(),
        rev,
        "no new picture within the same second"
    );

    a.tick_ms(start + 3500);
    let later = a.buttons_view().prompt.expect("prompt").1;
    assert!(later < first, "{later} should be below {first}");
    keymap::capture_cancel();
}

#[test]
fn nothing_else_acts_while_waiting_for_a_key() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);

    press(&mut a, Btn::B);
    press(&mut a, Btn::Down);
    assert!(matches!(a.phase(), Phase::Buttons), "B left while waiting");
    assert_eq!(
        a.buttons_view().cursor,
        A_ROW,
        "the cursor moved while waiting"
    );
    keymap::capture_cancel();
}

#[test]
fn leaving_the_screen_while_waiting_stops_the_capture() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);

    // The lid closes while the prompt is up.
    a.apply(Action::LidClose);
    assert!(
        matches!(a.phase(), Phase::Doze { .. }),
        "the lid did not sleep the app: {:?}",
        a.phase()
    );
    a.tick_ms(a.now() + 1);
    assert!(
        !keymap::capture_offer(spare_key(), 1),
        "keys are still being swallowed after the screen was left"
    );
    keymap::capture_cancel();
}

#[test]
fn reset_restores_the_built_in_map() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (d, mut a) = on_buttons_screen();
    let default_a = keymap::keys_for(Btn::A);

    down_to(&mut a, A_ROW);
    press(&mut a, Btn::A);
    assert!(keymap::capture_offer(spare_key(), 1));
    a.tick_ms(a.now() + 1);
    assert!(keymap::customised());

    // Back to the top, then to the last row.
    for _ in 0..A_ROW {
        press(&mut a, Btn::Up);
    }
    press(&mut a, Btn::Right);
    for _ in 0..(RESET_ROW - 8) {
        press(&mut a, Btn::Down);
    }
    assert_eq!(a.buttons_view().cursor, RESET_ROW);
    press(&mut a, Btn::A);

    assert!(!keymap::customised());
    assert_eq!(keymap::keys_for(Btn::A), default_a);
    assert!(a.buttons_view().notice.contains("restored"));
    let file = std::fs::read_to_string(d.path().join("Config/buttons.ini")).expect("buttons.ini");
    assert!(
        !file
            .lines()
            .any(|l| l.trim().chars().next().is_some_and(|c| c.is_ascii_digit())),
        "reset left bindings in the file:\n{file}"
    );
}

#[test]
fn left_and_right_move_between_the_columns_and_b_goes_back() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    press(&mut a, Btn::Right);
    assert_eq!(a.buttons_view().cursor, 8);
    press(&mut a, Btn::Left);
    assert_eq!(a.buttons_view().cursor, 0);
    press(&mut a, Btn::Up);
    assert_eq!(a.buttons_view().cursor, 0, "the cursor ran off the top");

    press(&mut a, Btn::B);
    assert_eq!(a.quick_menu(), Some(QuickRow::Buttons));
}

#[test]
fn opening_the_screen_again_starts_clean() {
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let (_d, mut a) = on_buttons_screen();
    down_to(&mut a, 6);
    press(&mut a, Btn::B);
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::Buttons));
    let view = a.buttons_view();
    assert_eq!(view.cursor, 0);
    assert!(view.notice.is_empty());
}
