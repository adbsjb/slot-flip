//! The drawn game cartridges: the NES Game Pak and the SNES cart.
//!
//! Drawn rather than cut from a bitmap, like the disc: a plastic slab with ridged
//! grips, a recessed label, and the narrower section at the bottom where the
//! connector is. Nothing on one is a logo; the label carries the game's cover art,
//! or its colour and title.
//!
//! Each is a [`Design`], so a new system's cartridge is a set of numbers here and
//! not new drawing code. A pak goes in no wider than the slot is, so it is scaled
//! down to `SEATED_W` as it goes in just as the other carts are.

use std::sync::OnceLock;

use crate::cart::{ink, label_colour, CartFace};
use crate::text;

/// Which cartridge a system is drawn as.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Pak {
    Nes,
    Snes,
}

/// A rounded rectangle: x, y, w, h, radius.
type Rr = (f32, f32, f32, f32, f32);
type Rect = (f32, f32, f32, f32);

pub struct Design {
    pub w: u32,
    pub h: u32,
    /// The label recess, in pixels: x, y, w, h. Cover art is fitted to exactly this.
    pub label: (u32, u32, u32, u32),
    /// The main slab, and the narrower connector section that steps in below it.
    body: Rr,
    foot: Rr,
    /// Where the slab meets the foot, for the ridge of shadow across the step.
    step_y: f32,
    plastic: [f32; 3],
    /// The ridged grips: grooves every 8 px down each.
    grips: &'static [Rect],
    /// A shallow recessed strip under the label.
    strip: Rr,
}

const NES: Design = Design {
    w: 300,
    h: 332,
    label: (44, 40, 212, 170),
    body: (0.0, 0.0, 300.0, 292.0, 15.0),
    foot: (30.0, 268.0, 240.0, 64.0, 7.0),
    step_y: 292.0,
    plastic: [154.0, 153.0, 149.0],
    grips: &[(12.0, 22.0, 28.0, 112.0), (260.0, 22.0, 28.0, 112.0)],
    strip: (62.0, 232.0, 176.0, 30.0, 6.0),
};

/// Wider than a Game Pak and shorter, a little lighter, with its ridges down both
/// sides of the label.
const SNES: Design = Design {
    w: 340,
    h: 282,
    label: (56, 30, 228, 168),
    body: (0.0, 0.0, 340.0, 246.0, 15.0),
    foot: (36.0, 226.0, 268.0, 56.0, 7.0),
    step_y: 246.0,
    plastic: [166.0, 165.0, 161.0],
    grips: &[(14.0, 34.0, 26.0, 150.0), (300.0, 34.0, 26.0, 150.0)],
    strip: (76.0, 208.0, 188.0, 22.0, 6.0),
};

impl Pak {
    pub const ALL: [Pak; 2] = [Pak::Nes, Pak::Snes];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn design(self) -> &'static Design {
        match self {
            Pak::Nes => &NES,
            Pak::Snes => &SNES,
        }
    }

    /// The label as fractions of the cart, for a badge (the favourite star).
    pub fn label_fraction(self) -> (f32, f32, f32, f32) {
        let d = self.design();
        (
            d.label.0 as f32 / d.w as f32,
            d.label.1 as f32 / d.h as f32,
            d.label.2 as f32 / d.w as f32,
            d.label.3 as f32 / d.h as f32,
        )
    }

    /// The cart's own coverage, for its shadow and for clipping.
    pub fn mask(self) -> &'static [u8] {
        static MASKS: [OnceLock<Vec<u8>>; 2] = [OnceLock::new(), OnceLock::new()];
        MASKS[self.index()].get_or_init(|| {
            let d = self.design();
            let mut out = Vec::with_capacity((d.w * d.h) as usize);
            for y in 0..d.h {
                for x in 0..d.w {
                    let dist = d.outline(x as f32 + 0.5, y as f32 + 0.5);
                    out.push((cover_at(dist) * 255.0).round() as u8);
                }
            }
            out
        })
    }

    pub fn face(self, title: &str, art: Option<Vec<u8>>) -> CartFace {
        self.design().face(title, art)
    }
}

const PAD: u32 = 6;
const MAX_LINES: usize = 3;
const MIN_PX: f32 = 10.0;

/// Signed distance to a rounded rectangle: negative inside.
fn rounded(x: f32, y: f32, r: Rr) -> f32 {
    let (rx, ry, rw, rh, rad) = r;
    let (cx, cy) = (rx + rw / 2.0, ry + rh / 2.0);
    let (qx, qy) = (
        (x - cx).abs() - (rw / 2.0 - rad),
        (y - cy).abs() - (rh / 2.0 - rad),
    );
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - rad
}

fn cover_at(d: f32) -> f32 {
    (0.5 - d).clamp(0.0, 1.0)
}

/// Fine deterministic grain, so the plastic is not a flat fill.
fn grain(x: u32, y: u32) -> f32 {
    let h = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
    let h = (h ^ (h >> 13)).wrapping_mul(0x5bd1_e995);
    (((h >> 16) & 7) as f32 - 3.5) * 0.9
}

fn in_rect(x: f32, y: f32, r: Rect) -> bool {
    x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

impl Design {
    fn outline(&self, x: f32, y: f32) -> f32 {
        rounded(x, y, self.body).min(rounded(x, y, self.foot))
    }

    /// How much darker (negative) or lighter (positive) the moulding makes a point
    /// of the plastic, in 0..255 steps.
    fn moulding(&self, x: f32, y: f32) -> f32 {
        let mut v = 0.0;
        for &(gx, gy, gw, gh) in self.grips {
            if in_rect(x, y, (gx, gy, gw, gh)) {
                match ((y - gy) as i32).rem_euclid(8) {
                    0..=2 => v -= 34.0,
                    3 => v += 16.0,
                    _ => {}
                }
            }
        }
        // The label recess: shadowed along its top and left, lit along its bottom
        // and right, as a step down into the plastic would be.
        let (lx, ly, lw, lh) = (
            self.label.0 as f32 - 4.0,
            self.label.1 as f32 - 4.0,
            self.label.2 as f32 + 8.0,
            self.label.3 as f32 + 8.0,
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
        let strip = rounded(x, y, self.strip);
        if strip < 0.0 {
            v -= 20.0;
        } else if strip < 1.6 {
            v += 14.0;
        }
        // The shadow the slab throws across the step down to the connector.
        let (fx, _, fw, _, _) = self.foot;
        if y > self.step_y - 7.0 && y < self.step_y + 1.0 && x > fx && x < fx + fw {
            v -= 26.0 * (1.0 - ((self.step_y - y) / 7.0).clamp(0.0, 1.0));
        }
        v
    }

    fn face(&self, title: &str, art: Option<Vec<u8>>) -> CartFace {
        let (lx, ly, lw, lh) = self.label;
        let (w, h) = (self.w, self.h);
        let label = match art {
            Some(a) if a.len() == (lw * lh * 4) as usize => a,
            _ => self.generated(title),
        };
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let d = self.outline(fx, fy);
                let cover = cover_at(d);
                if cover <= 0.0 {
                    continue;
                }
                let i = ((y * w + x) * 4) as usize;
                let on_label = (lx..lx + lw).contains(&x) && (ly..ly + lh).contains(&y);
                let mut c = if on_label {
                    let l = (((y - ly) * lw + (x - lx)) * 4) as usize;
                    [label[l] as f32, label[l + 1] as f32, label[l + 2] as f32]
                } else {
                    // Lit from above: lighter at the top, darker down the face.
                    let t = fy / h as f32;
                    let shade = 14.0 - 34.0 * t + grain(x, y) + self.moulding(fx, fy);
                    [
                        self.plastic[0] + shade,
                        self.plastic[1] + shade,
                        self.plastic[2] + shade,
                    ]
                };
                // A bright lip on the top edge, and the rim darker where it turns away.
                let inside = -d;
                if !on_label && inside < 2.2 {
                    let lit = if fy < h as f32 * 0.5 { 26.0 } else { -22.0 };
                    let k = 1.0 - inside.max(0.0) / 2.2;
                    for v in &mut c {
                        *v += lit * k;
                    }
                }
                rgba[i] = c[0].clamp(0.0, 255.0).round() as u8;
                rgba[i + 1] = c[1].clamp(0.0, 255.0).round() as u8;
                rgba[i + 2] = c[2].clamp(0.0, 255.0).round() as u8;
                rgba[i + 3] = (cover * 255.0).round() as u8;
            }
        }
        CartFace { rgba, w, h }
    }

    /// A label for a game with no cover art: its colour, a printed border, and its
    /// title across the middle.
    fn generated(&self, title: &str) -> Vec<u8> {
        let (_, _, lw, lh) = self.label;
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
}
