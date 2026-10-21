//! Renders the PlayStation slot as the slot draws it: the back strip, a disc part
//! way in, then the front plate over it, dark and light, and the plugged state.
//!
//!     cargo run -p slot-ui --example ps1_slot_preview -- out.png [covers_dir]

use slot_store::scan;
use slot_ui::{cart_face, disc_glare, ps1_slot, CartFace, DISC_D, SLOT_H, SLOT_W};

fn layer(canvas: &mut [f32], top: i32, strip: &[u8]) {
    for y in 0..SLOT_H as i32 {
        for x in 0..SLOT_W as i32 {
            let s = ((y as u32 * SLOT_W + x as u32) * 4) as usize;
            let a = strip[s + 3] as f32 / 255.0;
            if a <= 0.0 {
                continue;
            }
            let d = (((top + y) as u32 * SLOT_W + x as u32) * 3) as usize;
            for c in 0..3 {
                canvas[d + c] = strip[s + c] as f32 * a + canvas[d + c] * (1.0 - a);
            }
        }
    }
}

fn disc(canvas: &mut [f32], face: &CartFace, glare: &CartFace, left: i32, top: i32, size: i32) {
    for y in 0..size {
        for x in 0..size {
            let (cy, cx) = (top + y, left + x);
            if cy < 0
                || cy >= canvas.len() as i32 / (SLOT_W as i32 * 3)
                || cx < 0
                || cx >= SLOT_W as i32
            {
                continue;
            }
            let (fx, fy) = (
                (x * face.w as i32 / size) as u32,
                (y * face.h as i32 / size) as u32,
            );
            let i = ((fy * face.w + fx) * 4) as usize;
            let a = face.rgba[i + 3] as f32 / 255.0;
            let ga = glare.rgba[i + 3] as f32 / 255.0;
            let d = ((cy as u32 * SLOT_W + cx as u32) * 3) as usize;
            for c in 0..3 {
                let v = canvas[d + c] * (1.0 - a) + face.rgba[i + c] as f32 * a;
                canvas[d + c] = v * (1.0 - ga) + 255.0 * ga;
            }
        }
    }
}

fn cover_face(covers: Option<&std::path::Path>) -> CartFace {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("Games/PS1")).unwrap();
    let name = "Sky Racer";
    std::fs::write(
        root.path().join("Games/PS1").join(format!("{name}.cue")),
        "",
    )
    .unwrap();
    if let Some(dir) = covers {
        if let Some(png) = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "png"))
        {
            std::fs::create_dir_all(root.path().join("Labels/PS1")).unwrap();
            std::fs::copy(
                png,
                root.path().join("Labels/PS1").join(format!("{name}.png")),
            )
            .unwrap();
        }
    }
    cart_face(&scan(root.path()).unwrap()[0])
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("usage: ps1_slot_preview <out.png> [covers_dir]");
    let covers = std::env::args().nth(2);
    let face = cover_face(covers.as_deref().map(std::path::Path::new));
    let glare = disc_glare();
    let (w, h, gap) = (SLOT_W as usize, 150usize, 8usize);
    let top = 90i32;
    // (light theme, how far the disc's foot is below the top of the strip, plugged)
    let frames: [(bool, i32, bool); 8] = [
        (false, -26, false),
        (false, 12, false),
        (false, 30, false),
        (false, 44, false),
        (true, 12, false),
        (true, 30, false),
        (false, 0, true),
        (true, 0, true),
    ];
    let rows = frames.len();
    let sheet_h = rows * (h + gap) + gap;
    let mut sheet = vec![0x40u8; w * sheet_h * 3];
    for (r, (light, foot, plugged)) in frames.iter().enumerate() {
        let strips = ps1_slot(*light);
        let bg = if *light {
            [0xc9u8, 0xd1, 0xde]
        } else {
            [0x20, 0x24, 0x30]
        };
        let mut canvas: Vec<f32> = (0..w * h).flat_map(|_| bg.map(|c| c as f32)).collect();
        if *plugged {
            layer(&mut canvas, top, &strips.plugged);
        } else {
            layer(&mut canvas, top, &strips.back);
            let size = (DISC_D * 4 / 5) as i32;
            disc(
                &mut canvas,
                &face,
                &glare,
                (w as i32 - size) / 2,
                top + foot - size,
                size,
            );
            layer(&mut canvas, top, &strips.front);
        }
        let oy = gap + r * (h + gap);
        for y in 0..h {
            for x in 0..w {
                let s = (y * w + x) * 3;
                let d = ((oy + y) * w + x) * 3;
                for c in 0..3 {
                    sheet[d + c] = canvas[s + c].clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w as u32, sheet_h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&sheet)
        .unwrap();
    println!("wrote {out}");
}
