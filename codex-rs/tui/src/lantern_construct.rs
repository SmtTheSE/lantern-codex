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

/// Delay between one letter starting to form and the next.
const REVEAL_STEP_MS: u64 = 35;
/// Time each `░▒▓` stage of a forming letter lasts.
const REVEAL_STAGE_MS: u64 = 45;

/// How long `reveal` takes to finish a phrase of `chars` characters.
pub(crate) fn reveal_duration_ms(chars: usize) -> u64 {
    chars as u64 * REVEAL_STEP_MS + 3 * REVEAL_STAGE_MS + FLASH_MS
}

/// A phrase materializing letter by letter: each letter forms through `░▒▓`,
/// then locks in with a flash of the ring's gold. `ms` is time since the phrase
/// appeared; once `reveal_duration_ms` has passed it is plain `text` in `color`.
pub(crate) fn reveal(text: &str, ms: u64, color: Rgb) -> Vec<(char, Rgb)> {
    text.chars()
        .enumerate()
        .map(|(i, ch)| {
            let start = i as u64 * REVEAL_STEP_MS;
            if ch == ' ' || ms < start {
                return (' ', color);
            }
            let age = ms - start;
            match age / REVEAL_STAGE_MS {
                0 => ('░', lerp(FORMING, color, 0.3)),
                1 => ('▒', lerp(FORMING, color, 0.6)),
                2 => ('▓', lerp(FORMING, color, 0.85)),
                _ => {
                    let since = age - 3 * REVEAL_STAGE_MS;
                    let flash = 1.0 - (since as f32 / FLASH_MS as f32).clamp(0.0, 1.0);
                    (ch, lerp(color, glint(), flash * 0.9))
                }
            }
        })
        .collect()
}

pub(crate) const RING_COLS: usize = 6;
pub(crate) const RING_ROWS: usize = 3;
/// Time the orbiting spark takes to advance one pixel.
const ORBIT_STEP_MS: u64 = 70;
/// Pixels of the little ring's band, clockwise from the top left.
const ORBIT: [(usize, usize); 16] = [
    (1, 0), (2, 0), (3, 0), (4, 0),
    (5, 1), (5, 2), (5, 3), (5, 4),
    (4, 5), (3, 5), (2, 5), (1, 5),
    (0, 4), (0, 3), (0, 2), (0, 1),
];

/// A small ring that charges while the model works: a spark orbits the band
/// leaving a fading trail, and the hollow pulses with light (when the terminal
/// background is known). `RING_ROWS` rows of `RING_COLS` cells.
pub(crate) fn mini_ring(elapsed: Duration, bg: Option<Rgb>) -> Vec<Vec<crate::lantern_logo::Cell>> {
    let ms = elapsed.as_millis() as u64;
    let head = (ms / ORBIT_STEP_MS) as usize % ORBIT.len();
    let pulse = 0.5 + 0.5 * (ms as f32 / 450.0).sin();
    let pixel = |x: usize, y: usize| -> Option<Rgb> {
        if let Some(index) = ORBIT.iter().position(|&p| p == (x, y)) {
            let behind = (head + ORBIT.len() - index) % ORBIT.len();
            if behind == 0 {
                return Some(glint());
            }
            let trail = (1.0 - behind as f32 / 9.0).clamp(0.0, 1.0);
            let base = lerp((0x0B, 0x4D, 0x28), SOLID, trail);
            return Some(lerp(base, GLINT_GREEN, trail * trail * 0.6));
        }
        if (1..=4).contains(&x) && (1..=4).contains(&y) {
            return bg.map(|bg| blend(SOLID, bg, 0.06 + 0.10 * pulse));
        }
        None
    };
    (0..RING_ROWS)
        .map(|row| {
            (0..RING_COLS)
                .map(|col| crate::lantern_logo::compose(pixel(col, 2 * row), pixel(col, 2 * row + 1)))
                .collect()
        })
        .collect()
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

    #[test]
    fn reveal_forms_letters_left_to_right_then_settles() {
        let color = (0, 230, 77);
        let text = "Shaping…";
        let start = reveal(text, 0, color);
        assert_eq!(start[0].0, '░', "first letter begins forming at once");
        assert_eq!(start[3].0, ' ', "later letters have not started");
        let mid = reveal(text, 150, color);
        assert_eq!(mid[0].0, 'S', "first letter has locked in");
        assert!("░▒▓".contains(mid[3].0), "fourth letter is still forming: {}", mid[3].0);
        let done = reveal(text, reveal_duration_ms(text.chars().count()), color);
        assert_eq!(done.iter().map(|(c, _)| *c).collect::<String>(), text);
        assert!(done.iter().all(|(_, c)| *c == color), "flash has faded");
    }

    #[test]
    fn reveal_keeps_spaces_so_words_do_not_jump() {
        let shown: String = reveal("a b", 10_000, (1, 2, 3)).iter().map(|(c, _)| *c).collect();
        assert_eq!(shown, "a b");
    }

    #[test]
    fn mini_ring_has_fixed_size_and_a_hollow() {
        let ring = mini_ring(Duration::from_millis(0), None);
        assert_eq!(ring.len(), RING_ROWS);
        assert!(ring.iter().all(|row| row.len() == RING_COLS));
        assert_eq!(ring[1][2].ch, ' ', "hollow is empty without a known background");
        let lit = mini_ring(Duration::from_millis(0), Some((12, 12, 16)));
        assert!(lit[1][2].fg.is_some(), "hollow glows when the background is known");
    }

    #[test]
    fn mini_ring_spark_orbits_with_a_trail() {
        let at = |ms| mini_ring(Duration::from_millis(ms), None);
        assert_ne!(at(0), at(ORBIT_STEP_MS), "the spark moves");
        assert_eq!(at(0), at(ORBIT_STEP_MS * ORBIT.len() as u64), "one lap repeats");
        // The head is the gold/glint pixel and the pixel behind it is dimmer.
        let frame = at(0);
        let head = frame[0][1].fg.expect("head pixel");
        let behind = frame[0][4].fg.expect("tail pixel");
        let luma = |c: Rgb| 77 * u32::from(c.0) + 150 * u32::from(c.1) + 29 * u32::from(c.2);
        assert!(luma(head) > luma(behind), "{head:?} vs {behind:?}");
    }
}
