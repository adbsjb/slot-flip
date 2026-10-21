mod common;

use std::sync::mpsc;
use std::time::Duration;

use common::StubSnapshot;
use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{write_slot_state, Platform, SlotState};
use slot_ui::QuickRow;

/// Runs `body` on its own thread and fails if it does not finish: a menu that
/// loops forever would otherwise hang the whole suite instead of failing.
fn within_seconds<T: Send + 'static>(secs: u64, body: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(body());
    });
    rx.recv_timeout(Duration::from_secs(secs))
        .expect("it never finished: something is looping")
}

fn playing_disc() -> (tempfile::TempDir, App) {
    let d = tempfile::tempdir().unwrap();
    for sub in slot::root::DIRS {
        std::fs::create_dir(d.path().join(sub)).unwrap();
    }
    std::fs::create_dir_all(d.path().join("Games/PS1")).unwrap();
    std::fs::write(d.path().join("Games/PS1/Sky Racer.cue"), "").unwrap();
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Sky Racer".into()),
            cart_platform: Some(Platform::Ps1),
            clock_set: true,
            utc_offset_min: 0,
            ..Default::default()
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(StubSnapshot::boxed());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    assert_eq!(a.playing_platform(), Platform::Ps1);
    (d, a)
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn drawn(a: &App) -> usize {
    let mut out = Vec::new();
    a.draw(&mut out);
    out.len()
}

#[test]
fn every_menu_row_can_be_asked_for_its_value_while_a_disc_is_playing() {
    within_seconds(20, || {
        let (_d, a) = playing_disc();
        for row in QuickRow::ALL {
            let _ = a.quick_value(row);
        }
    });
}

#[test]
fn a_disc_has_no_shader_bezel_or_colour_correction_rows() {
    let (_d, a) = playing_disc();
    let hidden = a.in_game_hidden();
    for row in [
        QuickRow::Shader,
        QuickRow::ColourCorrection,
        QuickRow::Bezel,
    ] {
        assert!(hidden.contains(&row), "{row:?} is offered for a disc");
    }
}

#[test]
fn a_cart_still_gets_its_shader_row() {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let a = common::app_playing_in(d.path(), "Emerald");
    let hidden = a.in_game_hidden();
    assert!(!hidden.contains(&QuickRow::Shader));
    assert!(!hidden.contains(&QuickRow::ColourCorrection));
}

#[test]
fn the_in_game_menu_can_be_walked_and_every_setting_changed_over_a_disc() {
    within_seconds(60, || {
        let (_d, mut a) = playing_disc();
        a.apply(Action::QuickMenu);
        assert!(a.in_game_menu().is_some());
        for page in 0..3 {
            for _ in 0..20 {
                for btn in [Btn::Right, Btn::Left, Btn::Down] {
                    press(&mut a, btn);
                    a.update(1.0 / 60.0);
                    let _ = drawn(&a);
                }
                if a.in_game_menu().is_none() {
                    return;
                }
            }
            // Down into the next page: Settings, then Screen.
            let to = [QuickRow::Settings, QuickRow::PlayScreen][page.min(1)];
            for _ in 0..12 {
                if a.in_game_menu() == Some(to) {
                    break;
                }
                press(&mut a, Btn::Down);
            }
            if a.in_game_menu() == Some(to) {
                press(&mut a, Btn::A);
            }
        }
        assert!(
            matches!(a.phase(), Phase::Playing { .. }),
            "{:?}",
            a.phase()
        );
    });
}
