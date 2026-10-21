//! The slot a PlayStation disc goes into, drawn as a PlayStation.
//!
//! Same contract as the pre-drawn strips in `assets/`: a back layer that sits
//! behind the disc, a front layer that hides its lower part so it seems to go in
//! (with an opening cut where it passes), and a "plugged" layer with the opening
//! closed. The opening keeps the width, depth and position of the cart slot's so
//! the insertion clips the same way; only its shape changes, to a round-edged
//! lid opening.
//!
//! The console is its plastic: that mid grey, a lit top edge and a darker lower
//! lip, a fine grain, a bevel where the lid is cut away, and the POWER, RESET and
//! OPEN buttons with the power light. Drawn rather than traced, so it needs no
//! binary asset and is covered by tests.

use crate::bitmaps::{SLOT_H, SLOT_W};
use crate::text;

const W: u32 = SLOT_W;
const H: u32 = SLOT_H;

/// The opening, matched to the cart slot's: 254 wide about the centre, 44 deep.
const OPEN_HALF: f32 = 127.0;
const OPEN_DEPTH: f32 = 44.0;
const CX: f32 = 359.5;

type Rgb = [u8; 3];

pub struct Ps1Slot {
    pub back: Vec<u8>,
    pub front: Vec<u8>,
    pub plugged: Vec<u8>,
}

struct Palette {
    plastic: Rgb,
    lit: Rgb,
    lip: Rgb,
    button: Rgb,
    ink: Rgb,
}

fn palette(light: bool) -> Palette {
    if light {
        Palette {
            plastic: [0xb9, 0xb8, 0xb4],
            lit: [0xe2, 0xe1, 0xdd],
            lip: [0x8f, 0x8e, 0x8b],
            button: [0xc6, 0xc5, 0xc1],
            ink: [0x58, 0x57, 0x55],
        }
    } else {
        Palette {
            plastic: [0xa2, 0xa1, 0x9d],
            lit: [0xcd, 0xcc, 0xc8],
            lip: [0x76, 0x75, 0x72],
            button: [0xb1, 0xb0, 0xac],
            ink: [0x4a, 0x49, 0x47],
        }
    }
}

/// How deep the opening is at column `x`: zero outside it, and a half ellipse
/// across it, so the walls stand straight at the edges and curve to a floor.
fn open_depth(x: u32) -> f32 {
    let u = (x as f32 + 0.5 - CX) / OPEN_HALF;
    if u.abs() >= 1.0 {
        return 0.0;
    }
    OPEN_DEPTH * (1.0 - u * u).sqrt()
}

fn in_opening(x: u32, y: u32) -> bool {
    (y as f32 + 0.5) < open_depth(x)
}

/// Fine deterministic grain, so the plastic is not a flat fill.
fn grain(x: u32, y: u32) -> i32 {
    let h = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
    let h = (h ^ (h >> 13)).wrapping_mul(0x5bd1_e995);
    ((h >> 16) & 7) as i32 - 3
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
}

fn shade(c: Rgb, by: i32) -> Rgb {
    [0, 1, 2].map(|i| (c[i] as i32 + by).clamp(0, 255) as u8)
}

struct Img {
    rgba: Vec<u8>,
}

impl Img {
    fn new() -> Img {
        Img {
            rgba: vec![0; (W * H * 4) as usize],
        }
    }

    fn put(&mut self, x: u32, y: u32, c: Rgb, a: u8) {
        if x < W && y < H {
            let i = ((y * W + x) * 4) as usize;
            self.rgba[i..i + 3].copy_from_slice(&c);
            self.rgba[i + 3] = a;
        }
    }

    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * W + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }

    /// `c` laid over what is there, `a` of the way.
    fn blend(&mut self, x: u32, y: u32, c: Rgb, a: f32) {
        if x >= W || y >= H || a <= 0.0 {
            return;
        }
        let p = self.at(x, y);
        let m = mix([p[0], p[1], p[2]], c, a);
        self.put(x, y, m, p[3].max((a * 255.0) as u8));
    }
}

/// Coverage of a rounded rectangle at a pixel, with a soft one pixel edge.
fn rounded(x: f32, y: f32, rect: (f32, f32, f32, f32), r: f32) -> f32 {
    let (rx, ry, rw, rh) = rect;
    let (cx, cy) = (rx + rw / 2.0, ry + rh / 2.0);
    let (qx, qy) = (
        (x - cx).abs() - (rw / 2.0 - r),
        (y - cy).abs() - (rh / 2.0 - r),
    );
    let d = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;
    (0.5 - d).clamp(0.0, 1.0)
}

/// The plate: plastic across the whole strip, optionally with the opening cut.
fn plate(p: &Palette, opening: bool) -> Img {
    let mut img = Img::new();
    for y in 0..H {
        for x in 0..W {
            if opening && in_opening(x, y) {
                continue;
            }
            // Lit along the top, darker down the face, a lip along the bottom.
            let t = y as f32 / (H - 1) as f32;
            let mut c = mix(p.lit, p.plastic, (y as f32 / 3.0).min(1.0));
            c = mix(c, p.lip, ((t - 0.62) / 0.38).max(0.0).powi(2));
            c = shade(c, grain(x, y));
            img.put(x, y, c, 255);
        }
    }
    if opening {
        bevel_opening(&mut img, p);
    }
    img
}

/// The edge where the lid is cut away: a bright lip, then the wall falling into
/// shadow.
fn bevel_opening(img: &mut Img, p: &Palette) {
    const REACH: i32 = 5;
    let transparent = |x: i32, y: i32| {
        x >= 0 && y >= 0 && (x as u32) < W && (y as u32) < H && in_opening(x as u32, y as u32)
    };
    for y in 0..H {
        for x in 0..W {
            if in_opening(x, y) {
                continue;
            }
            let mut nearest = f32::MAX;
            for dy in -REACH..=REACH {
                for dx in -REACH..=REACH {
                    if transparent(x as i32 + dx, y as i32 + dy) {
                        nearest = nearest.min(((dx * dx + dy * dy) as f32).sqrt());
                    }
                }
            }
            if nearest > REACH as f32 {
                continue;
            }
            if nearest < 1.6 {
                img.blend(x, y, p.lit, 0.55);
            } else {
                img.blend(
                    x,
                    y,
                    [0x2b, 0x2a, 0x29],
                    0.34 * (1.0 - (nearest - 1.6) / 3.4),
                );
            }
        }
    }
}

/// A raised rounded button with its label.
fn button(img: &mut Img, p: &Palette, rect: (f32, f32, f32, f32), label: &str) {
    let (bx, by, bw, bh) = rect;
    for y in (by as u32 - 2)..(by as u32 + bh as u32 + 3) {
        for x in (bx as u32 - 2)..(bx as u32 + bw as u32 + 3) {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            // The shadow the button casts down and to the right.
            let drop = rounded(fx - 1.0, fy - 1.5, rect, 4.0);
            img.blend(x, y, [0x30, 0x2f, 0x2e], 0.32 * drop);
            let body = rounded(fx, fy, rect, 4.0);
            if body > 0.0 {
                let t = ((fy - by) / bh).clamp(0.0, 1.0);
                let c = mix(shade(p.button, 14), shade(p.button, -16), t);
                img.blend(x, y, c, body);
            }
        }
    }
    if let Some(font) = text::label_font() {
        let (tw, th) = ((bw - 6.0) as u32, (bh - 4.0) as u32);
        let layout = text::fit_box(font, label, tw as f32, th as f32, 1, th as f32, 6.0);
        let cover = text::coverage(tw, th, &layout);
        let (ox, oy) = (bx as u32 + 3, by as u32 + 2);
        for ty in 0..th {
            for tx in 0..tw {
                let a = cover[(ty * tw + tx) as usize];
                if a > 0 {
                    img.blend(ox + tx, oy + ty, p.ink, a as f32 / 255.0 * 0.9);
                }
            }
        }
    }
}

/// The power light: a green lamp in a small bezel.
fn led(img: &mut Img, cx: f32, cy: f32) {
    for y in (cy as u32 - 6)..(cy as u32 + 7) {
        for x in (cx as u32 - 6)..(cx as u32 + 7) {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            img.blend(
                x,
                y,
                [0x35, 0x34, 0x33],
                (1.0 - (d - 4.5).max(0.0)).clamp(0.0, 1.0) * 0.8,
            );
            if d < 3.2 {
                let glow = 1.0 - d / 3.2;
                img.blend(
                    x,
                    y,
                    mix([0x2a, 0xa8, 0x3a], [0xa6, 0xff, 0xb0], glow * 0.7),
                    1.0,
                );
            } else if d < 6.0 {
                img.blend(x, y, [0x58, 0xd0, 0x6a], 0.22 * (1.0 - (d - 3.2) / 2.8));
            }
        }
    }
}

/// Everything the console wears on its face.
fn dress(img: &mut Img, p: &Palette) {
    // POWER and its light on the left, RESET and OPEN on the right, clear of the
    // opening and inside what a 640 px panel shows of the 720 px strip.
    led(img, 138.0, 29.0);
    button(img, p, (152.0, 20.0, 50.0, 17.0), "POWER");
    button(img, p, (520.0, 20.0, 46.0, 17.0), "RESET");
    button(img, p, (574.0, 20.0, 42.0, 17.0), "OPEN");
}

/// The lid, seen closed: a round ring let into the plate, shown when the slot is
/// plugged.
fn lid_ring(img: &mut Img, p: &Palette) {
    let (cx, cy, r) = (CX, 27.0, 21.0);
    for y in 0..H {
        for x in 0..W {
            let (fx, fy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let d = (fx * fx + fy * fy).sqrt();
            if (d - r).abs() < 3.0 {
                // A groove: shadowed on its upper-left wall, lit on its lower-right.
                let lit = (fx + fy) / (2.0 * r);
                let k = (1.0 - (d - r).abs() / 3.0) * 0.5;
                img.blend(x, y, if lit > 0.0 { p.lit } else { [0x2f, 0x2e, 0x2d] }, k);
            } else if d < r - 3.0 {
                // The lid's centre sits a shade lower than the plate around it.
                img.blend(x, y, p.lip, 0.10);
            }
        }
    }
}

/// What is behind the opening: the dark inside of the console, deepening
/// downwards.
fn back(p: &Palette) -> Img {
    let mut img = Img::new();
    for y in 0..H {
        for x in 0..W {
            let t = y as f32 / (H - 1) as f32;
            let base = mix([0x2a, 0x2a, 0x2c], [0x0c, 0x0c, 0x0e], t.min(1.0).sqrt());
            img.put(x, y, shade(base, grain(x, y) / 2), 255);
        }
    }
    // A hint of the lid's rim catching the light along the top of the interior.
    for x in 0..W {
        img.blend(x, 0, p.lip, 0.5);
        img.blend(x, 1, p.lip, 0.22);
    }
    img
}

pub fn ps1_slot(light: bool) -> Ps1Slot {
    let p = palette(light);

    let mut front = plate(&p, true);
    dress(&mut front, &p);

    let mut plugged = plate(&p, false);
    dress(&mut plugged, &p);
    lid_ring(&mut plugged, &p);

    Ps1Slot {
        back: back(&p).rgba,
        front: front.rgba,
        plugged: plugged.rgba,
    }
}

/// Whether the opening is cut at this column of the strip, for tests and callers
/// that line something up with it.
pub fn opening_depth(x: u32) -> f32 {
    open_depth(x)
}
