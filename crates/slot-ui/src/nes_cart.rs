//! The NES Game Pak.
//!
//! Drawn rather than cut from a bitmap, like the disc: a grey slab with the two
//! ridged grips at the top, a recessed label, and the narrower section at the
//! bottom where the connector is. Nothing on it is a logo; the label carries the
//! game's cover art, or its colour and title.
//!
//! Portrait, as the real one is, and no wider than the slot is, so it is scaled
//! down to `SEATED_W` as it goes in just as the other carts are.

use std::sync::OnceLock;

use crate::cart::{ink, label_colour, CartFace};
use crate::text;

pub const NES_W: u32 = 300;
pub const NES_H: u32 = 332;

/// The label recess, in pixels: x, y, w, h. Cover art is fitted to exactly this.
pub const LABEL_PX: (u32, u32, u32, u32) = (44, 40, 212, 170);

/// The label as fractions of the cart, for a badge (the favourite star).
pub const LABEL: (f32, f32, f32, f32) = (
    LABEL_PX.0 as f32 / NES_W as f32,
    LABEL_PX.1 as f32 / NES_H as f32,
    LABEL_PX.2 as f32 / NES_W as f32,
    LABEL_PX.3 as f32 / NES_H as f32,
);

/// The main slab, and the narrower connector section that steps in below it.
const BODY: (f32, f32, f32, f32, f32) = (0.0, 0.0, 300.0, 292.0, 15.0);
const FOOT: (f32, f32, f32, f32, f32) = (30.0, 268.0, 240.0, 64.0, 7.0);
/// Where the slab meets the foot, for the ridge of shadow across the step.
const STEP_Y: f32 = 292.0;

const PLASTIC: [f32; 3] = [154.0, 153.0, 149.0];

const PAD: u32 = 6;
const MAX_LINES: usize = 3;
const MIN_PX: f32 = 10.0;

/// Signed distance to a rounded rectangle: negative inside.
fn rounded(x: f32, y: f32, r: (f32, f32, f32, f32, f32)) -> f32 {
    let (rx, ry, rw, rh, rad) = r;
    let (cx, cy) = (rx + rw / 2.0, ry + rh / 2.0);
    let (qx, qy) = (
        (x - cx).abs() - (rw / 2.0 - rad),
        (y - cy).abs() - (rh / 2.0 - rad),
    );
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - rad
}

fn outline(x: f32, y: f32) -> f32 {
    rounded(x, y, BODY).min(rounded(x, y, FOOT))
}

fn cover_at(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// The cart's own coverage, for its shadow and for clipping.
pub fn mask() -> &'static [u8] {
    static MASK: OnceLock<Vec<u8>> = OnceLock::new();
    MASK.get_or_init(|| {
        let mut out = Vec::with_capacity((NES_W * NES_H) as usize);
        for y in 0..NES_H {
            for x in 0..NES_W {
                let d = outline(x as f32 + 0.5, y as f32 + 0.5);
                out.push((cover_at(d) * 255.0).round() as u8);
            }
        }
        out
    })
}

/// Fine deterministic grain, so the plastic is not a flat fill.
fn grain(x: u32, y: u32) -> f32 {
    let h = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
    let h = (h ^ (h >> 13)).wrapping_mul(0x5bd1_e995);
    (((h >> 16) & 7) as f32 - 3.5) * 0.9
}

fn in_rect(x: f32, y: f32, r: (f32, f32, f32, f32)) -> bool {
    x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

/// How much darker (negative) or lighter (positive) the moulding makes a point
/// of the plastic, in 0..255 steps.
fn moulding(x: f32, y: f32) -> f32 {
    let mut v = 0.0;
    // The two ridged grips at the top corners: grooves every 8 px.
    for gx in [12.0f32, 260.0] {
        if in_rect(x, y, (gx, 22.0, 28.0, 112.0)) {
            match ((y - 22.0) as i32).rem_euclid(8) {
                0..=2 => v -= 34.0,
                3 => v += 16.0,
                _ => {}
            }
        }
    }
    // The label recess: shadowed along its top and left, lit along its bottom
    // and right, as a step down into the plastic would be.
    let (lx, ly, lw, lh) = (
        LABEL_PX.0 as f32 - 4.0,
        LABEL_PX.1 as f32 - 4.0,
        LABEL_PX.2 as f32 + 8.0,
        LABEL_PX.3 as f32 + 8.0,
    );
    if in_rect(x, y, (lx, ly, lw, lh)) {
        let (top, left) = (y - ly < 4.0, x - lx < 4.0);
        let (bottom, right) = (ly + lh - y <= 4.0, lx + lw - x <= 4.0);
        if top || left {
            v -= 38.0;
        } else if bottom || right {
            v += 20.0;
        }
    }
    // A shallow recessed strip under the label.
    let strip = rounded(x, y, (62.0, 232.0, 176.0, 30.0, 6.0));
    if strip < 0.0 {
        v -= 20.0;
    } else if strip < 1.6 {
        v += 14.0;
    }
    // The shadow the slab throws across the step down to the connector.
    if y > STEP_Y - 7.0 && y < STEP_Y + 1.0 && x > 30.0 && x < 270.0 {
        v -= 26.0 * (1.0 - ((STEP_Y - y) / 7.0).clamp(0.0, 1.0));
    }
    v
}

pub fn face(title: &str, art: Option<Vec<u8>>) -> CartFace {
    let (lx, ly, lw, lh) = LABEL_PX;
    let label = match art {
        Some(a) if a.len() == (lw * lh * 4) as usize => a,
        _ => generated(title),
    };
    let mut rgba = vec![0u8; (NES_W * NES_H * 4) as usize];
    for y in 0..NES_H {
        for x in 0..NES_W {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let d = outline(fx, fy);
            let cover = cover_at(d);
            if cover <= 0.0 {
                continue;
            }
            let i = ((y * NES_W + x) * 4) as usize;
            let on_label = (lx..lx + lw).contains(&x) && (ly..ly + lh).contains(&y);
            let mut c = if on_label {
                let l = (((y - ly) * lw + (x - lx)) * 4) as usize;
                [label[l] as f32, label[l + 1] as f32, label[l + 2] as f32]
            } else {
                // Lit from above: lighter at the top, darker down the face.
                let t = fy / NES_H as f32;
                let shade = 14.0 - 34.0 * t + grain(x, y) + moulding(fx, fy);
                [PLASTIC[0] + shade, PLASTIC[1] + shade, PLASTIC[2] + shade]
            };
            // A bright lip on the top edge, and the rim darker where it turns away.
            let inside = -d;
            if !on_label {
                if inside < 2.2 {
                    let lit = if fy < NES_H as f32 * 0.5 { 26.0 } else { -22.0 };
                    let k = 1.0 - inside.max(0.0) / 2.2;
                    for v in &mut c {
                        *v += lit * k;
                    }
                }
            }
            rgba[i] = c[0].clamp(0.0, 255.0).round() as u8;
            rgba[i + 1] = c[1].clamp(0.0, 255.0).round() as u8;
            rgba[i + 2] = c[2].clamp(0.0, 255.0).round() as u8;
            rgba[i + 3] = (cover * 255.0).round() as u8;
        }
    }
    CartFace {
        rgba,
        w: NES_W,
        h: NES_H,
    }
}

/// A label for a game with no cover art: its colour, a printed border, and its
/// title across the middle.
fn generated(title: &str) -> Vec<u8> {
    let (_, _, lw, lh) = LABEL_PX;
    let bg = label_colour(title);
    let ink = ink(bg);
    let mut rgba = Vec::with_capacity((lw * lh * 4) as usize);
    for y in 0..lh {
        for x in 0..lw {
            let edge = x.min(lw - 1 - x).min(y).min(lh - 1 - y);
            // A thin printed border a little way in.
            let line = if (7..=8).contains(&edge) { 0.35 } else { 0.0 };
            for c in 0..3 {
                rgba.push((ink[c] as f32 * line + bg[c] as f32 * (1.0 - line)).round() as u8);
            }
            rgba.push(255);
        }
    }
    if let Some(font) = text::label_font() {
        let (bw, bh) = (lw - 2 * 18, lh - 2 * 18);
        let layout = text::fit_box(
            font,
            title,
            (bw - 2 * PAD) as f32,
            (bh - 2 * PAD) as f32,
            MAX_LINES,
            (bh - 2 * PAD) as f32 / MAX_LINES as f32,
            MIN_PX,
        );
        let mut block = vec![0u8; (bw * bh * 4) as usize];
        for px in block.chunks_exact_mut(4) {
            px.copy_from_slice(&[bg[0], bg[1], bg[2], 255]);
        }
        text::draw_centred(&mut block, bw, bh, &layout, ink);
        for row in 0..bh {
            let src = (row * bw * 4) as usize;
            let dst = (((18 + row) * lw + 18) * 4) as usize;
            rgba[dst..dst + (bw * 4) as usize]
                .copy_from_slice(&block[src..src + (bw * 4) as usize]);
        }
    }
    rgba
}
