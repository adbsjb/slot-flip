mod common;

use slot::app::{App, Phase};
use slot_store::{write_slot_state, Core, Platform, SlotState};
use slot_ui::Toast;

fn root_with(platform: Platform, core_installed: bool) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    for sub in slot::root::DIRS {
        std::fs::create_dir_all(d.path().join(sub)).unwrap();
    }
    let dir = d.path().join("Games").join(platform.dir_name());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("Chrono.{}", platform.extensions()[0])),
        vec![0u8; 0x400],
    )
    .unwrap();
    if core_installed {
        let core = Core::default_for(platform);
        std::fs::write(
            d.path().join("System").join(slot::core::dylib_name(core)),
            "",
        )
        .unwrap();
    }
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Chrono".into()),
            cart_platform: Some(platform),
            clock_set: true,
            utc_offset_min: 0,
            ..Default::default()
        },
    )
    .unwrap();
    d
}

fn run(d: &tempfile::TempDir) -> (App, bool) {
    let mut a = App::boot(d.path());
    a.set_snapshot(common::StubSnapshot::boxed());
    a.on_core_ready();
    let mut said = false;
    for _ in 0..120 {
        a.update(1.0 / 60.0);
        said |= a.toast() == Some(Toast::NeedsAddon);
    }
    (a, said)
}

#[test]
fn only_the_super_nintendo_core_is_in_the_add_on_pack() {
    for core in Core::EVERY {
        assert_eq!(core.is_addon(), core == Core::Snes9x, "{core:?}");
    }
}

#[test]
fn a_game_whose_core_is_in_the_add_on_pack_says_so_when_the_pack_is_missing() {
    let d = root_with(Platform::Snes, false);
    let (a, said) = run(&d);
    assert!(
        !matches!(a.phase(), Phase::Playing { .. } | Phase::Inserting { .. }),
        "it started without a core: {:?}",
        a.phase()
    );
    assert!(said, "the player was never told why nothing started");
    assert!(!slot::core::installed(d.path(), Core::Snes9x));
}

#[test]
fn the_same_game_starts_once_the_pack_is_installed() {
    let d = root_with(Platform::Snes, true);
    let (a, said) = run(&d);
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    assert!(!said, "it asked for a pack that is installed");
    assert!(slot::core::installed(d.path(), Core::Snes9x));
}

#[test]
fn the_cores_in_the_main_download_are_never_asked_for() {
    for platform in [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Ps1,
        Platform::Nes,
    ] {
        let d = root_with(platform, false);
        assert!(
            slot::core::installed(d.path(), Core::default_for(platform)),
            "{platform:?} was treated as needing an add-on"
        );
    }
}
