use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(names, vec!["GBA", "GB", "GBC", "PS1", "NES"]);
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [
            Platform::Gba,
            Platform::Gb,
            Platform::Gbc,
            Platform::Ps1,
            Platform::Nes
        ]
    );
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert!(
            !Platform::ALL[..i].contains(p),
            "{p:?} is named twice in the ring, so two shelves would hold one platform"
        );
    }
}

#[test]
fn each_platform_takes_only_its_own_extensions() {
    assert!(Platform::Gba.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gbc")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(Platform::Gba.accepts(std::path::Path::new("Shrek.GBA")));
}

#[test]
fn a_playstation_takes_disc_images_and_playlists_but_not_their_track_files() {
    let ps1 = Platform::Ps1;
    for ok in ["Crash.cue", "Crash.CHD", "FF7.m3u", "Game.pbp"] {
        assert!(ps1.accepts(std::path::Path::new(ok)), "{ok}");
    }
    // A cue sheet's .bin is a track of a game already on the shelf, not a game.
    for no in ["Crash.bin", "Crash.iso", "Metroid Fusion.gba", "Tetris.gb"] {
        assert!(!ps1.accepts(std::path::Path::new(no)), "{no}");
    }
    assert!(!Platform::Gba.accepts(std::path::Path::new("Crash.cue")));
}

#[test]
fn each_platform_runs_only_the_cores_that_can() {
    use slot_store::Core;
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc] {
        assert!(Core::Mgba.runs(p), "{p:?}");
        assert!(!Core::Pcsx.runs(p), "{p:?}");
    }
    assert!(Core::Gpsp.runs(Platform::Gba));
    assert!(!Core::Gpsp.runs(Platform::Gb));
    assert!(Core::Pcsx.runs(Platform::Ps1));
    assert!(!Core::Mgba.runs(Platform::Ps1), "mGBA cannot play a disc");
    assert!(!Core::Gpsp.runs(Platform::Ps1));
    assert_eq!(Core::default_for(Platform::Ps1), Core::Pcsx);
}

#[test]
fn a_nes_takes_its_own_files_and_runs_on_fceumm_alone() {
    use slot_store::Core;
    for ok in ["Contra.nes", "Zelda.NES", "Disk.fds", "Hack.unf"] {
        assert!(Platform::Nes.accepts(std::path::Path::new(ok)), "{ok}");
    }
    for no in ["Contra.sfc", "Tetris.gb", "Crash.cue", "Fusion.gba"] {
        assert!(!Platform::Nes.accepts(std::path::Path::new(no)), "{no}");
    }
    assert!(!Platform::Gba.accepts(std::path::Path::new("Contra.nes")));
    assert!(Core::Fceumm.runs(Platform::Nes));
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc, Platform::Ps1] {
        assert!(!Core::Fceumm.runs(p), "{p:?}");
    }
    for core in [Core::Mgba, Core::Gpsp, Core::Pcsx] {
        assert!(!core.runs(Platform::Nes), "{core:?} cannot play a NES game");
    }
    assert_eq!(Core::default_for(Platform::Nes), Core::Fceumm);
    assert_eq!(Core::parse("fceumm"), Some(Core::Fceumm));
    assert_eq!(Core::parse(Core::Fceumm.as_str()), Some(Core::Fceumm));
    assert_eq!(Platform::Nes.picture(), (256, 240));
}

#[test]
fn only_the_game_boy_family_has_a_screen_to_shade_and_a_gba_canvas_to_sit_on() {
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc] {
        assert!(p.has_screen_options(), "{p:?}");
        assert!(!p.native_video(), "{p:?}");
    }
    for p in [Platform::Ps1, Platform::Nes] {
        assert!(!p.has_screen_options(), "{p:?}");
        assert!(p.native_video(), "{p:?}");
    }
}
