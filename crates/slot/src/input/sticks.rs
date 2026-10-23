//! The Flip's two analog sticks, for a game that reads them.
//!
//! The input thread that owns the gamepad node writes the raw axes here; the
//! session reads them once per update and hands them to the core. They are
//! kept apart from the button events because a stick is a position, not a
//! press, and a game wants the whole range.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_RX: u16 = 0x03;
pub const ABS_RY: u16 = 0x04;

/// The full swing of an axis, as the Flip's gamepad node reports it (+-32767).
pub const FULL: i32 = 32_767;

/// How far a stick must be pushed before it counts. The Flip's sticks do not
/// rest at zero (a right stick at rest was measured at about -5100 and 1400),
/// so without this a character would walk on its own.
pub const DEAD_ZONE: i32 = 7_000;

static AXES: [AtomicI32; 4] = [
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
];

/// Whether the right stick is the game's, not the frontend's. It stands in for
/// the R2 button otherwise (see `evdev::Stick`), and cannot be both.
static TAKEN: AtomicBool = AtomicBool::new(false);

/// Left x, left y, right x, right y: what a core is handed.
pub type Sticks = [i16; 4];

/// A raw axis event from the gamepad node. Anything that is not a stick is ignored.
pub fn feed(code: u16, value: i32) {
    let at = match code {
        ABS_X => 0,
        ABS_Y => 1,
        ABS_RX => 2,
        ABS_RY => 3,
        _ => return,
    };
    AXES[at].store(value.clamp(-FULL, FULL), Ordering::Relaxed);
}

/// The sticks as a game should see them, dead zone applied.
pub fn read() -> Sticks {
    let axis = |i: usize| AXES[i].load(Ordering::Relaxed);
    let (lx, ly) = dead_zone(axis(0), axis(1));
    let (rx, ry) = dead_zone(axis(2), axis(3));
    [lx, ly, rx, ry]
}

/// Forget where the sticks were, for a core that has just been given none.
pub fn release() {
    for a in &AXES {
        a.store(0, Ordering::Relaxed);
    }
}

pub fn set_taken(taken: bool) {
    TAKEN.store(taken, Ordering::Relaxed);
}

pub fn taken() -> bool {
    TAKEN.load(Ordering::Relaxed)
}

/// A circular dead zone: nothing inside it, and the rest of the range stretched
/// back over the full swing so a stick pushed to the edge still reads as the edge.
/// Circular rather than per axis, so a diagonal is not dead on one axis and live
/// on the other.
pub fn dead_zone(x: i32, y: i32) -> (i16, i16) {
    let (fx, fy) = (f64::from(x), f64::from(y));
    let mag = fx.hypot(fy);
    let dead = f64::from(DEAD_ZONE);
    if mag <= dead {
        return (0, 0);
    }
    let full = f64::from(FULL);
    let scaled = ((mag - dead) / (full - dead)).min(1.0) * full;
    let k = scaled / mag;
    ((fx * k).round() as i16, (fy * k).round() as i16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The axes are global, so the tests that write them take turns.
    static AXES_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn a_stick_at_rest_reads_as_still() {
        assert_eq!(dead_zone(0, 0), (0, 0));
        // What the Flip's right stick measured at rest.
        assert_eq!(dead_zone(-5096, 1404), (0, 0));
    }

    #[test]
    fn the_dead_zone_is_round_not_square() {
        // Each axis alone is inside the zone; together they are past it.
        let edge = (f64::from(DEAD_ZONE) * 0.8) as i32;
        let (x, y) = dead_zone(edge, edge);
        assert!(x > 0 && y > 0, "{x} {y}");
        // And the same push along one axis is not.
        assert_eq!(dead_zone(edge, 0), (0, 0));
    }

    #[test]
    fn the_edge_of_the_stick_is_still_the_edge() {
        assert_eq!(dead_zone(FULL, 0), (i16::MAX, 0));
        assert_eq!(dead_zone(0, -FULL), (0, -i16::MAX));
        let (x, y) = dead_zone(FULL, FULL);
        let mag = f64::from(x).hypot(f64::from(y));
        assert!((mag - f64::from(FULL)).abs() < 2.0, "{mag}");
    }

    #[test]
    fn a_push_just_past_the_zone_barely_moves_and_one_more_moves_further() {
        let (a, _) = dead_zone(DEAD_ZONE + 1, 0);
        let (b, _) = dead_zone(DEAD_ZONE + 5_000, 0);
        let (c, _) = dead_zone(DEAD_ZONE + 15_000, 0);
        assert!(0 < a && a < 10, "{a}");
        assert!(a < b && b < c, "{a} {b} {c}");
    }

    #[test]
    fn direction_is_kept() {
        let (x, y) = dead_zone(-20_000, 10_000);
        assert!(x < 0 && y > 0);
        // The ratio between the two survives the stretch.
        let want = 10_000.0 / 20_000.0;
        assert!((f64::from(y) / f64::from(-x) - want).abs() < 0.01);
    }

    #[test]
    fn an_axis_event_lands_on_the_stick_it_belongs_to() {
        let _g = AXES_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        release();
        feed(ABS_X, 30_000);
        feed(ABS_RY, -30_000);
        feed(2, 255); // the L2 trigger is not a stick
        let [lx, ly, rx, ry] = read();
        assert!(
            lx > 20_000 && ly == 0 && rx == 0 && ry < -20_000,
            "{lx} {ly} {rx} {ry}"
        );
        release();
        assert_eq!(read(), [0; 4]);
    }

    #[test]
    fn a_value_past_the_range_is_clamped() {
        let _g = AXES_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        release();
        feed(ABS_X, 90_000);
        assert_eq!(read()[0], i16::MAX);
        release();
    }
}
