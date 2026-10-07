//! Animated "construct" badge for the working row.
//!
//! Eight cells assemble one by one through `░▒▓█` stages, each flashing
//! white-green as it locks in. Once the construct is whole, a sparkle sweeps
//! across it; then the pieces dissolve right to left and the cycle restarts.
//! The function is pure (time in, cells out) so it is easy to test and the
//! caller only has to ask for the next frame.

use std::time::Duration;

use crate::color::blend;

pub(crate) type Rgb = (u8, u8, u8);

/// One badge cell: a glyph and its foreground color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cell {
    pub glyph: char,
    pub color: Rgb,
}

pub(crate) const CELLS: usize = 8;

/// Full cycle length.
pub(crate) const PERIOD_MS: u64 = 3600;
/// Delay between one cell starting to form and the next.
const BUILD_STEP_MS: u64 = 200;
/// Time one cell takes to go from empty to solid.
const FORM_MS: u64 = 500;
/// How long the lock-in flash fades.
const FLASH_MS: u64 = 300;
/// When the finished construct starts holding (sparkle sweep).
const HOLD_START_MS: u64 = 2000;
/// When the pieces start dissolving.
const DISSOLVE_START_MS: u64 = 2900;
const DISSOLVE_STEP_MS: u64 = 50;
const DISSOLVE_MS: u64 = 350;

const FORMING: Rgb = (0x12, 0x4A, 0x2A);
const SOLID: Rgb = crate::lantern::RING_GREEN;
const GLINT_GREEN: Rgb = (0xE8, 0xFF, 0xF0);

/// Lock-in flash and sparkle color: the ring's gold in the heartland theme.
fn glint() -> Rgb {
    if crate::lantern::heartland() { crate::lantern::RING_GOLD } else { GLINT_GREEN }
}

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    blend(b, a, t.clamp(0.0, 1.0))
}

fn glyph_for(progress: f32) -> char {
    match progress {
        p if p < 0.08 => '·',
        p if p < 0.30 => '░',
        p if p < 0.55 => '▒',
        p if p < 0.80 => '▓',
        _ => '█',
    }
}

/// Cheap deterministic hash for twinkles, so frames are reproducible.
fn twinkle(bucket: u64, cell: usize) -> bool {
    let h = bucket.wrapping_mul(2_654_435_761) ^ (cell as u64).wrapping_mul(40_503);
    (h >> 7) % 13 == 0
}

/// The badge `elapsed` into the animation.
pub(crate) fn badge(elapsed: Duration) -> [Cell; CELLS] {
    let ms = elapsed.as_millis() as u64;
    let t = ms % PERIOD_MS;
    let twinkle_bucket = t / 120;
    let sweep = (HOLD_START_MS..DISSOLVE_START_MS).contains(&t).then(|| {
        (t - HOLD_START_MS) as f32 / (DISSOLVE_START_MS - HOLD_START_MS) as f32
            * (CELLS as f32 + 2.0)
            - 1.0
    });

    std::array::from_fn(|i| {
        let build_start = i as u64 * BUILD_STEP_MS;
        let progress = if t < DISSOLVE_START_MS {
            ((t.saturating_sub(build_start)) as f32 / FORM_MS as f32).clamp(0.0, 1.0)
        } else {
            let start = DISSOLVE_START_MS + (CELLS as u64 - 1 - i as u64) * DISSOLVE_STEP_MS;
            1.0 - ((t.saturating_sub(start)) as f32 / DISSOLVE_MS as f32).clamp(0.0, 1.0)
        };

        let mut glyph = glyph_for(progress);
        let mut color = lerp(FORMING, SOLID, progress);

        // Lock-in flash right after this cell finishes forming.
        let done_at = build_start + FORM_MS;
        if t >= done_at && t < done_at + FLASH_MS {
            let flash = 1.0 - (t - done_at) as f32 / FLASH_MS as f32;
            color = lerp(color, glint(), flash * 0.9);
        }

        if progress >= 1.0 {
            // Sparkle sweep across the finished construct.
            if let Some(sweep) = sweep {
                let distance = (i as f32 - sweep).abs();
                if distance < 1.5 {
                    color = lerp(color, glint(), (1.0 - distance / 1.5) * 0.75);
                    if distance < 0.5 {
                        glyph = '✦';
                    }
                }
            }
            // Occasional twinkle while whole.
            if glyph == '█' && twinkle(twinkle_bucket, i) {
                glyph = '✦';
                color = lerp(color, glint(), 0.6);
            }
        }
        Cell { glyph, color }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(ms: u64) -> [Cell; CELLS] {
        badge(Duration::from_millis(ms))
    }

    #[test]
    fn starts_empty_and_ends_dissolved() {
        assert!(at(0).iter().all(|c| c.glyph == '·'));
        assert!(at(PERIOD_MS - 1).iter().all(|c| c.glyph == '·'));
    }

    #[test]
    fn builds_left_to_right() {
        let cells = at(900);
        assert_eq!(cells[0].glyph, '█', "first piece is already solid");
        assert_eq!(cells[CELLS - 1].glyph, '·', "last piece has not started");
        let order = |c: &Cell| "·░▒▓█".find(c.glyph).unwrap_or(4);
        assert!(
            cells.windows(2).all(|w| order(&w[0]) >= order(&w[1])),
            "progress should not increase left to right: {cells:?}"
        );
    }

    #[test]
    fn whole_construct_holds_with_a_sparkle() {
        let cells = at(2500);
        assert!(cells.iter().all(|c| matches!(c.glyph, '█' | '✦')), "{cells:?}");
        assert!(cells.iter().any(|c| c.glyph == '✦'), "sweep should leave a sparkle");
    }

    #[test]
    fn dissolves_right_to_left() {
        let cells = at(DISSOLVE_START_MS + 150);
        assert_ne!(cells[CELLS - 1].glyph, '█', "last piece goes first");
        assert_eq!(cells[0].glyph, '█', "first piece is still solid");
    }

    #[test]
    fn repeats_every_period() {
        for ms in [0, 321, 1500, 2600, 3300] {
            assert_eq!(at(ms), at(ms + PERIOD_MS), "ms={ms}");
        }
    }

    #[test]
    fn lock_in_flash_is_brighter_than_solid() {
        // Cell 0 finishes forming at FORM_MS, so just after it should glint.
        let flash = at(FORM_MS + 5)[0].color;
        let settled = at(FORM_MS + FLASH_MS + 5)[0].color;
        assert!(flash.0 > settled.0, "{flash:?} vs {settled:?}");
    }
}
