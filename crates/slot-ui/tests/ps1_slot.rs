use slot_ui::{ps1_opening_depth, ps1_slot, slot_front, SLOT_H, SLOT_W};

fn alpha(strip: &[u8], x: u32, y: u32) -> u8 {
    strip[((y * SLOT_W + x) * 4 + 3) as usize]
}

fn rgb(strip: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * SLOT_W + x) * 4) as usize;
    [strip[i], strip[i + 1], strip[i + 2]]
}

#[test]
fn every_strip_is_the_size_of_the_cart_slots() {
    for light in [false, true] {
        let s = ps1_slot(light);
        let n = (SLOT_W * SLOT_H * 4) as usize;
        assert_eq!((s.back.len(), s.front.len(), s.plugged.len()), (n, n, n));
    }
}

#[test]
fn the_opening_is_where_the_cart_slots_is_so_a_disc_clips_the_same_way() {
    let cart = slot_front();
    let cart_cols: Vec<u32> = (0..SLOT_W).filter(|&x| alpha(cart, x, 2) < 128).collect();
    let ours: Vec<u32> = (0..SLOT_W)
        .filter(|&x| ps1_opening_depth(x) > 2.0)
        .collect();
    let (c0, c1) = (cart_cols[0] as i32, *cart_cols.last().unwrap() as i32);
    let (o0, o1) = (ours[0] as i32, *ours.last().unwrap() as i32);
    assert!(
        (c0 - o0).abs() <= 6 && (c1 - o1).abs() <= 6,
        "cart {c0}..{c1}, ps1 {o0}..{o1}"
    );
    assert!(ps1_opening_depth(360) > 40.0 && ps1_opening_depth(360) <= 44.0);
}

#[test]
fn the_front_is_cut_away_at_the_opening_and_solid_everywhere_else() {
    for light in [false, true] {
        let f = ps1_slot(light).front;
        assert_eq!(
            alpha(&f, 360, 20),
            0,
            "the opening is clear so the disc shows"
        );
        assert_eq!(alpha(&f, 360, 2), 0);
        for x in [0, 60, 150, 600, 700] {
            assert_eq!(alpha(&f, x, 10), 255, "plate at column {x}");
        }
        assert_eq!(alpha(&f, 360, SLOT_H - 3), 255, "the lip under the opening");
    }
}

#[test]
fn the_back_is_solid_and_dark_and_the_plugged_slot_is_closed() {
    for light in [false, true] {
        let s = ps1_slot(light);
        for (x, y) in [(10, 5), (360, 20), (700, 50)] {
            assert_eq!(alpha(&s.back, x, y), 255);
            let [r, g, b] = rgb(&s.back, x, y);
            assert!(
                r < 90 && g < 90 && b < 90,
                "the inside is dark: {r},{g},{b}"
            );
        }
        for (x, y) in [(360, 2), (360, 30), (10, 5), (700, 50)] {
            assert_eq!(alpha(&s.plugged, x, y), 255, "closed at ({x},{y})");
        }
    }
}

#[test]
fn the_console_is_grey_plastic_with_a_green_power_light() {
    let f = ps1_slot(false).front;
    let [r, g, b] = rgb(&f, 60, 30);
    assert!(
        r.abs_diff(g) < 12 && g.abs_diff(b) < 12,
        "plastic is neutral: {r},{g},{b}"
    );
    assert!((90..210).contains(&r), "mid grey: {r}");
    let [lr, lg, lb] = rgb(&f, 138, 29);
    assert!(
        lg > lr + 40 && lg > lb + 40,
        "the power light is green: {lr},{lg},{lb}"
    );
}

#[test]
fn the_light_theme_is_a_lighter_console() {
    let (d, l) = (ps1_slot(false).front, ps1_slot(true).front);
    let sum = |s: &[u8]| rgb(s, 60, 30).iter().map(|&c| c as u32).sum::<u32>();
    assert!(sum(&l) > sum(&d));
}

#[test]
fn the_controls_clear_the_part_of_the_strip_a_panel_shows() {
    // The panel is 640 px of a 720 px strip, centred: 40..680.
    let f = ps1_slot(false).front;
    let ink = |x0: u32, x1: u32| {
        (x0..x1)
            .any(|x| (15..40).any(|y| rgb(&f, x, y) != rgb(&f, x0.saturating_sub(30).max(1), 10)))
    };
    assert!(ink(150, 205), "POWER is drawn");
    assert!(ink(520, 616), "RESET and OPEN are drawn");
}

#[test]
fn drawing_it_twice_gives_the_same_pixels() {
    assert_eq!(ps1_slot(false).front, ps1_slot(false).front);
    assert_eq!(ps1_slot(true).plugged, ps1_slot(true).plugged);
}
