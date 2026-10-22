use slot_store::{scan, Cart, Platform};
use slot_ui::{
    cart_box, cart_face, gb_cart_shadow, gb_shell_of, is_clear, label_rect, label_size, seated_box,
    CartFace, GbShell, MOUTH_W,
};
use tempfile::TempDir;

fn tmp_root() -> TempDir {
    let d = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(d.path().join("Games/NES")).expect("games dir");
    d
}

fn pak(d: &TempDir, name: &str) -> Cart {
    std::fs::write(d.path().join("Games/NES").join(format!("{name}.nes")), "").expect("rom");
    scan(d.path())
        .unwrap()
        .into_iter()
        .find(|c| c.stem == name)
        .expect("the pak")
}

fn write_art(d: &TempDir, name: &str, w: u32, h: u32, rgb: [u8; 3]) {
    std::fs::create_dir_all(d.path().join("Labels/NES")).unwrap();
    let f = std::fs::File::create(d.path().join("Labels/NES").join(format!("{name}.png"))).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let px: Vec<u8> = (0..w * h).flat_map(|_| rgb).collect();
    enc.write_header().unwrap().write_image_data(&px).unwrap();
}

fn at(face: &CartFace, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * face.w + x) * 4) as usize;
    [
        face.rgba[i],
        face.rgba[i + 1],
        face.rgba[i + 2],
        face.rgba[i + 3],
    ]
}

#[test]
fn a_nes_game_is_a_drawn_pak_and_never_a_gba_or_game_boy_cart() {
    let d = tmp_root();
    let c = pak(&d, "Contra");
    assert_eq!(c.platform, Platform::Nes);
    assert_eq!(gb_shell_of(&c), Some(GbShell::Nes));
    assert!(!is_clear(&c), "a Pak is solid plastic, never see-through");
}

#[test]
fn the_pak_is_portrait_and_goes_in_no_wider_than_the_slot_is() {
    let (w, h) = cart_box(Platform::Nes);
    assert!(h > w, "a Game Pak is taller than it is wide: {w}x{h}");
    let (sw, sh) = seated_box(Platform::Nes);
    assert!(
        (sw as f32) < MOUTH_W,
        "{sw} is wider than the slot ({MOUTH_W})"
    );
    // Scaled evenly, so it is not squashed on the way in.
    assert!((sw as f32 / sh as f32 - w as f32 / h as f32).abs() < 0.01);
}

#[test]
fn the_face_is_the_size_of_its_box_and_clear_outside_its_outline() {
    let d = tmp_root();
    let f = cart_face(&pak(&d, "Contra"));
    assert_eq!((f.w, f.h), cart_box(Platform::Nes));
    assert_eq!(f.rgba.len(), (f.w * f.h * 4) as usize);
    // The corners are rounded away, and the foot steps in from the sides.
    assert_eq!(at(&f, 0, 0)[3], 0, "the top corner is square");
    assert_eq!(at(&f, 2, f.h - 3)[3], 0, "the foot is as wide as the slab");
    assert_eq!(at(&f, f.w - 3, f.h - 3)[3], 0);
    assert_eq!(at(&f, f.w / 2, f.h / 2)[3], 255);
    assert_eq!(at(&f, f.w / 2, f.h - 4)[3], 255, "the connector is missing");
    assert_eq!(at(&f, 150, 150)[3], 255);
}

#[test]
fn the_slab_is_grey_plastic_and_the_foot_is_narrower() {
    let d = tmp_root();
    let f = cart_face(&pak(&d, "Contra"));
    let [r, g, b, _] = at(&f, 150, 250);
    assert!(
        r.abs_diff(g) < 12 && g.abs_diff(b) < 12,
        "neutral plastic: {r},{g},{b}"
    );
    assert!((90..210).contains(&r), "mid grey: {r}");
    let width = |y: u32| (0..f.w).filter(|&x| at(&f, x, y)[3] > 128).count();
    assert!(
        width(150) > width(f.h - 6) + 40,
        "no step down to the connector"
    );
}

#[test]
fn the_grips_are_ridged() {
    let d = tmp_root();
    let f = cart_face(&pak(&d, "Contra"));
    let column: Vec<u8> = (22..134).map(|y| at(&f, 26, y)[0]).collect();
    let (lo, hi) = (*column.iter().min().unwrap(), *column.iter().max().unwrap());
    assert!(hi - lo > 30, "the grip is a flat patch: {lo}..{hi}");
}

#[test]
fn the_label_recess_holds_the_art_edge_to_edge() {
    let d = tmp_root();
    write_art(&d, "Contra", 120, 96, [200, 20, 20]);
    let c = pak(&d, "Contra");
    let (lw, lh) = label_size(&c);
    let f = cart_face(&c);
    let (x, y, w, h) = label_rect(Some(GbShell::Nes));
    let (px, py) = ((x * f.w as f32) as u32, (y * f.h as f32) as u32);
    for (dx, dy) in [(2, 2), (lw / 2, lh / 2), (lw - 3, lh - 3)] {
        let [r, g, b, a] = at(&f, px + dx, py + dy);
        assert!(
            r > 150 && g < 60 && b < 60 && a == 255,
            "({dx},{dy}) is {r},{g},{b}"
        );
    }
    assert!((w * f.w as f32) as u32 == lw && (h * f.h as f32) as u32 == lh);
}

#[test]
fn a_pak_with_no_art_is_coloured_by_its_title_and_the_title_is_printed() {
    let d = tmp_root();
    let a = cart_face(&pak(&d, "Sky Racer"));
    let b = cart_face(&pak(&d, "Midnight Garden"));
    assert_ne!(
        at(&a, 60, 50),
        at(&b, 60, 50),
        "two titles got the same label colour"
    );
    let bg = at(&a, 60, 50);
    let differs = |x: u32, y: u32| {
        let p = at(&a, x, y);
        (0..3).map(|c| p[c].abs_diff(bg[c]) as u32).sum::<u32>() > 90
    };
    let ink_rows = (50..200)
        .filter(|&y| (60..240).any(|x| differs(x, y)))
        .count();
    assert!(ink_rows > 10, "no title was printed on the label");
}

#[test]
fn art_that_is_the_wrong_size_for_the_label_is_refitted_not_smeared() {
    let d = tmp_root();
    write_art(&d, "Tall", 359, 512, [30, 160, 30]);
    let f = cart_face(&pak(&d, "Tall"));
    let (x, y, _, _) = label_rect(Some(GbShell::Nes));
    let [r, g, b, a] = at(
        &f,
        (x * f.w as f32) as u32 + 100,
        (y * f.h as f32) as u32 + 80,
    );
    assert!(g > 120 && r < 80 && b < 80 && a == 255, "{r},{g},{b}");
}

#[test]
fn the_shadow_is_the_paks_outline() {
    let s = gb_cart_shadow(GbShell::Nes);
    let (w, h) = cart_box(Platform::Nes);
    assert_eq!((s.w, s.h), (w, h));
    let alpha = |x: u32, y: u32| s.rgba[((y * s.w + x) * 4 + 3) as usize];
    assert_eq!(alpha(0, 0), 0);
    assert_eq!(alpha(s.w / 2, s.h / 2), 255);
}

#[test]
fn the_other_systems_are_unchanged_by_it() {
    assert_eq!(cart_box(Platform::Gba), (360, 212));
    assert_ne!(
        gb_shell_of(&Cart {
            platform: Platform::Gb,
            stem: "x".into(),
            rom: "Games/GB/x.gb".into(),
            label: None,
            code: String::new(),
            shell: None,
            title: String::new(),
        }),
        Some(GbShell::Nes)
    );
}
