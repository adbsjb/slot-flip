//! The "Buttons" info screen: a simple, scrollable list of the current button
//! bindings. The actual remapping lives in Slot/Config/buttons.ini; this screen
//! shows what is in effect and where to change it, so there is a visible menu
//! entry for mappable buttons without a heavyweight editor.

use slot_gfx::{Draw, TexId, OUT_H, OUT_W};

use crate::plate::UndoFace;
use crate::text;

pub const PANEL_W: u32 = 560;
pub const PANEL_H: u32 = 360;

const BG: [u8; 3] = [0x1a, 0x1a, 0x1d];
const HEAD_INK: [u8; 3] = [0xff, 0xff, 0xff];
const BODY_INK: [u8; 3] = [0xc8, 0xc8, 0xd0];
const DIM_INK: [u8; 3] = [0x8a, 0x8a, 0x94];

const MARGIN: f32 = 26.0;
const HEAD_PX: f32 = 22.0;
const BODY_PX: f32 = 16.0;
const SMALL_PX: f32 = 13.0;

fn print(px_buf: &mut [u8], w: u32, h: u32, x: f32, y: f32, s: &str, size: f32, c: [u8; 3]) -> f32 {
    let Some(font) = text::label_font() else {
        return y + size * 1.4;
    };
    let layout = text::fit(font, s, f32::MAX, 1, size, size);
    let measured = layout
        .lines
        .first()
        .map(|l| text::line_width(font, l, size, layout.tracking))
        .unwrap_or(0.0);
    let (bw, bh) = ((measured.ceil() as u32).max(1), (size * 1.6).ceil() as u32);
    let cov = text::coverage(bw, bh, &layout);
    for row in 0..bh {
        for col in 0..bw {
            let a = cov[(row * bw + col) as usize] as u32;
            if a == 0 {
                continue;
            }
            let (dx, dy) = (x as u32 + col, y as u32 + row);
            if dx >= w || dy >= h {
                continue;
            }
            let at = ((dy * w + dx) * 4) as usize;
            let inv = 255 - a;
            for (k, ink) in c.iter().enumerate() {
                let under = px_buf[at + k] as u32;
                px_buf[at + k] = ((*ink as u32 * a + under * inv) / 255) as u8;
            }
            px_buf[at + 3] = 255;
        }
    }
    y + size * 1.4
}

/// Build the Buttons screen face.
///
/// `custom` lines are the active user overrides as "<code> -> <button>"; empty
/// means the built-in defaults are in use.
pub fn buttons_face(custom: &[(u16, &'static str)]) -> UndoFace {
    let (w, h) = (PANEL_W, PANEL_H);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        px.extend_from_slice(&[BG[0], BG[1], BG[2], 240]);
    }

    let left = MARGIN;
    let mut y = MARGIN;
    y = print(&mut px, w, h, left, y, "BUTTONS", HEAD_PX, HEAD_INK);
    y += 8.0;

    if custom.is_empty() {
        y = print(
            &mut px,
            w,
            h,
            left,
            y,
            "Using the built-in button map for this device.",
            BODY_PX,
            BODY_INK,
        );
    } else {
        y = print(
            &mut px,
            w,
            h,
            left,
            y,
            "Active overrides (code -> button):",
            BODY_PX,
            BODY_INK,
        );
        y += 4.0;
        for (code, name) in custom.iter().take(10) {
            let line = format!("    {code}  ->  {}", name.to_uppercase());
            y = print(&mut px, w, h, left, y, &line, BODY_PX, BODY_INK);
        }
    }

    y += 16.0;
    for line in [
        "To remap, edit:",
        "    Slot/Config/buttons.ini",
        "",
        "Each line: <key-code> = <button>",
        "e.g.  109 = r1",
    ] {
        y = print(&mut px, w, h, left, y, line, SMALL_PX, DIM_INK);
    }

    UndoFace { rgba: px, w, h }
}

pub fn draw_buttons(face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    out.push(Draw::Tex {
        x: (OUT_W as f32 - PANEL_W as f32) / 2.0,
        y: (OUT_H as f32 - PANEL_H as f32) / 2.0,
        w: PANEL_W as f32,
        h: PANEL_H as f32,
        tex,
        alpha: 1.0,
    });
}
