use slot_retro::{ButtonMask, LibretroCore, RetroCore, GBA_H, GBA_W};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static CORE_LOCK: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    CORE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn scratch(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&d).expect("scratch dir");
    d
}

/// The device core, built by `cores/pcsx/build.sh`. Linux only, and absent
/// unless it has been built, so these tests skip quietly elsewhere.
fn core(dir: &Path) -> Option<LibretroCore> {
    let so = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pcsx_rearmed_libretro.so");
    if !so.exists() {
        eprintln!("no pcsx_rearmed core built, skipping");
        return None;
    }
    Some(
        LibretroCore::open_with(&so, dir, dir)
            .expect("the pcsx core is present but would not open"),
    )
}

/// A PlayStation executable that does nothing but loop: a header and two
/// instructions (`j` to itself, then a nop). Content the core can be pointed at
/// by path without it being anyone's game, and without needing a BIOS or disc.
fn spin_exe(dir: &Path) -> PathBuf {
    let put = |b: &mut Vec<u8>, at: usize, v: u32| b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    let mut exe = vec![0u8; 2048 * 2];
    exe[..8].copy_from_slice(b"PS-X EXE");
    put(&mut exe, 0x10, 0x8001_0000); // initial pc
    put(&mut exe, 0x18, 0x8001_0000); // load address
    put(&mut exe, 0x1c, 2048); // size of the text that follows the header
    put(&mut exe, 0x30, 0x801f_fff0); // initial sp
    let region = b"Sony Computer Entertainment Inc. for North America area";
    exe[0x4c..0x4c + region.len()].copy_from_slice(region);
    put(&mut exe, 2048, 0x0800_4000); // j 0x80010000
    put(&mut exe, 2052, 0); // nop
    let path = dir.join("spin.exe");
    std::fs::write(&path, exe).unwrap();
    path
}

#[test]
fn the_core_says_it_opens_discs_itself_by_path() {
    let _g = lock();
    let dir = scratch("pcsx-info");
    let Some(c) = core(&dir) else { return };
    let info = c.system_info();
    eprintln!("{info:?}");
    assert!(info.need_fullpath, "a disc core must be handed a path");
    for ext in ["cue", "chd", "m3u"] {
        assert!(
            info.extensions.iter().any(|e| e == ext),
            "no .{ext}: {info:?}"
        );
    }
}

#[test]
fn content_is_loaded_by_path_and_drawn_at_its_own_size() {
    let _g = lock();
    let dir = scratch("pcsx-disc");
    let Some(mut c) = core(&dir) else { return };
    let exe = spin_exe(&dir);

    c.set_native_video(true);
    let loaded = c.load(&exe);
    eprintln!("load: {loaded:?}");
    for _ in 0..120 {
        c.run_frame(ButtonMask::default());
    }
    let (w, h) = c.video_size();
    eprintln!(
        "video {w}x{h}, {} bytes, fps {:?}",
        c.video_xrgb8888().len(),
        c.av_info()
    );
    assert!(loaded.is_ok(), "{loaded:?}");
    assert_eq!(c.video_xrgb8888().len(), (w * h * 4) as usize);
    assert!(
        w > GBA_W || h > GBA_H,
        "{w}x{h} should be bigger than the GBA canvas, not cropped to it"
    );
}

#[test]
fn a_missing_disc_is_an_error_not_a_crash() {
    let _g = lock();
    let dir = scratch("pcsx-missing");
    let Some(mut c) = core(&dir) else { return };
    assert!(c.load(&dir.join("nothing-here.cue")).is_err());
}
