//! The "Buttons" screen: every remappable button and the key it is on, in two
//! columns. Pick a button, press the key you want for it, and the app rebinds it
//! and saves `Slot/Config/buttons.ini`. The screen only draws; what the keys do
//! lives in the app, so everything here is a plain view of data it is handed.

use slot_gfx::{Draw, TexId, OUT_H, OUT_W};

use crate::plate::UndoFace;
use crate::text;

pub const PANEL_W: u32 = 560;
pub const PANEL_H: u32 = 360;

const BG: [u8; 3] = [0x1a, 0x1a, 0x1d];
const HEAD_INK: [u8; 3] = [0xff, 0xff, 0xff];
const BODY_INK: [u8; 3] = [0xc8, 0xc8, 0xd0];
const DIM_INK: [u8; 3] = [0x8a, 0x8a, 0x94];
const NOTICE_INK: [u8; 3] = [0x9d, 0xd6, 0xa8];
const SELECT_BG: [u8; 3] = [0x2f, 0x4a, 0x86];
const PROMPT_BG: [u8; 3] = [0x26, 0x26, 0x2e];
const PROMPT_EDGE: [u8; 3] = [0x6f, 0x9c, 0xf0];

const MARGIN: f32 = 26.0;
const HEAD_PX: f32 = 22.0;
const BODY_PX: f32 = 16.0;
const SMALL_PX: f32 = 13.0;

const ROW_H: f32 = 24.0;
const COL_W: f32 = 252.0;
const KEYS_AT: f32 = 76.0;

/// One line of the screen: a button and the key it is on (empty for an action
/// row such as "Reset").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonsRow {
    pub label: String,
    pub keys: String,
}

/// What to draw. `prompt` is `(button, seconds left)` while the screen is
/// waiting for a key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ButtonsView {
    pub rows: Vec<ButtonsRow>,
    pub cursor: usize,
    pub prompt: Option<(String, u32)>,
    pub notice: String,
}

impl ButtonsView {
    /// Rows in the first column; the rest go in the second.
    pub fn split(&self) -> usize {
        self.rows.len().div_ceil(2)
    }
}

struct Canvas {
    px: Vec<u8>,
    w: u32,
    h: u32,
}

impl Canvas {
    fn new(w: u32, h: u32) -> Self {
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            px.extend_from_slice(&[BG[0], BG[1], BG[2], 240]);
        }
        Canvas { px, w, h }
    }

    fn fill(&mut self, x: f32, y: f32, rw: f32, rh: f32, c: [u8; 3]) {
        let (x0, y0) = (x.max(0.0) as u32, y.max(0.0) as u32);
        let (x1, y1) = (
            ((x + rw).max(0.0) as u32).min(self.w),
            ((y + rh).max(0.0) as u32).min(self.h),
        );
        for yy in y0..y1 {
            for xx in x0..x1 {
                let at = ((yy * self.w + xx) * 4) as usize;
                self.px[at..at + 3].copy_from_slice(&c);
                self.px[at + 3] = 255;
            }
        }
    }

    fn frame(&mut self, x: f32, y: f32, rw: f32, rh: f32, edge: f32, fill: [u8; 3], line: [u8; 3]) {
        self.fill(x, y, rw, rh, line);
        self.fill(x + edge, y + edge, rw - 2.0 * edge, rh - 2.0 * edge, fill);
    }

    fn print(&mut self, x: f32, y: f32, s: &str, size: f32, c: [u8; 3]) -> f32 {
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
                if dx >= self.w || dy >= self.h {
                    continue;
                }
                let at = ((dy * self.w + dx) * 4) as usize;
                let inv = 255 - a;
                for (k, ink) in c.iter().enumerate() {
                    let under = self.px[at + k] as u32;
                    self.px[at + k] = ((*ink as u32 * a + under * inv) / 255) as u8;
                }
                self.px[at + 3] = 255;
            }
        }
        y + size * 1.4
    }

    fn finish(self) -> UndoFace {
        UndoFace {
            rgba: self.px,
            w: self.w,
            h: self.h,
        }
    }
}

/// Build the Buttons screen face.
pub fn buttons_face(view: &ButtonsView) -> UndoFace {
    let mut cv = Canvas::new(PANEL_W, PANEL_H);

    let mut y = cv.print(MARGIN, MARGIN, "BUTTONS", HEAD_PX, HEAD_INK);
    y += 6.0;

    let split = view.split().max(1);
    for (i, row) in view.rows.iter().enumerate() {
        let (col, line) = (i / split, i % split);
        let x = MARGIN + col as f32 * (COL_W + 8.0);
        let top = y + line as f32 * ROW_H;
        let selected = i == view.cursor && view.prompt.is_none();
        if selected {
            cv.fill(x - 8.0, top - 1.0, COL_W, ROW_H - 2.0, SELECT_BG);
        }
        let (label_ink, keys_ink) = if selected {
            (HEAD_INK, HEAD_INK)
        } else {
            (BODY_INK, DIM_INK)
        };
        cv.print(x, top, &row.label, BODY_PX, label_ink);
        if !row.keys.is_empty() {
            cv.print(x + KEYS_AT, top, &row.keys, BODY_PX, keys_ink);
        }
    }

    let below = y + split as f32 * ROW_H + 10.0;
    if !view.notice.is_empty() {
        cv.print(MARGIN, below, &view.notice, BODY_PX, NOTICE_INK);
    }
    let mut hint = below + 30.0;
    for line in [
        "A  rebind        Left / Right  other column        B  back",
        "No key within a few seconds cancels. Reset restores the built-in map.",
    ] {
        hint = cv.print(MARGIN, hint, line, SMALL_PX, DIM_INK);
    }

    if let Some((button, secs)) = &view.prompt {
        let (bw, bh) = (400.0, 132.0);
        let (bx, by) = ((PANEL_W as f32 - bw) / 2.0, (PANEL_H as f32 - bh) / 2.0);
        cv.frame(bx, by, bw, bh, 2.0, PROMPT_BG, PROMPT_EDGE);
        let mut py = by + 16.0;
        py = cv.print(
            bx + 24.0,
            py,
            "Press the key you want for",
            BODY_PX,
            BODY_INK,
        );
        py = cv.print(
            bx + 24.0,
            py + 2.0,
            &button.to_uppercase(),
            HEAD_PX,
            HEAD_INK,
        );
        let wait = if *secs == 1 {
            "Cancels in 1 second".to_string()
        } else {
            format!("Cancels in {secs} seconds")
        };
        cv.print(bx + 24.0, py + 6.0, &wait, SMALL_PX, DIM_INK);
    }

    cv.finish()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn view(n: usize) -> ButtonsView {
        ButtonsView {
            rows: (0..n)
                .map(|i| ButtonsRow {
                    label: format!("B{i}"),
                    keys: "Space".into(),
                })
                .collect(),
            ..ButtonsView::default()
        }
    }

    #[test]
    fn the_first_column_takes_the_larger_half() {
        assert_eq!(view(16).split(), 8);
        assert_eq!(view(15).split(), 8);
        assert_eq!(view(1).split(), 1);
        assert_eq!(view(0).split(), 0);
    }

    #[test]
    fn the_face_is_always_the_panel_size() {
        let mut v = view(16);
        let plain = buttons_face(&v);
        assert_eq!((plain.w, plain.h), (PANEL_W, PANEL_H));
        assert_eq!(plain.rgba.len(), (PANEL_W * PANEL_H * 4) as usize);

        v.prompt = Some(("a".into(), 4));
        v.notice = "A is now Space".into();
        let prompting = buttons_face(&v);
        assert_eq!(prompting.rgba.len(), plain.rgba.len());
        assert_ne!(
            prompting.rgba, plain.rgba,
            "the prompt must change the picture"
        );
    }

    #[test]
    fn the_selected_row_is_highlighted() {
        let mut v = view(16);
        let before = buttons_face(&v).rgba;
        v.cursor = 3;
        let after = buttons_face(&v).rgba;
        assert_ne!(before, after, "moving the cursor must change the picture");
    }

    #[test]
    fn an_empty_view_does_not_panic() {
        let _ = buttons_face(&ButtonsView::default());
    }
}
