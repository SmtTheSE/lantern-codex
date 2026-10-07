//! Truecolor power ring for the startup banner.
//!
//! An octagonal emerald ring between two bars, like the reference terminal,
//! rendered rather than hand-drawn: the band is a bevelled tube lit from the
//! upper left (diffuse shading, a specular highlight, a cool rim light and an
//! engraved center groove), edges are anti-aliased by supersampling, and a soft
//! green glow spills over the terminal background. Two pixels stack into one
//! terminal cell with `▀`/`▄`, so the image is `WIDTH` columns by `ROWS` rows.
//! It is an original generic ring, not the Corps emblem.

use crate::color::blend;

pub(crate) type Rgb = (u8, u8, u8);

/// One terminal cell: a half block plus its colors. `None` colors are transparent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cell {
    pub ch: char,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
}

pub(crate) const WIDTH: usize = 28;
const HEIGHT: usize = 28;
pub(crate) const ROWS: usize = HEIGHT / 2;

const CENTER: (f32, f32) = (14.0, 14.0);
/// Apothem (center to flat side) of the octagon's outer and inner edges.
const OUTER: f32 = 11.4;
const INNER: f32 = 7.6;
/// Subsamples per pixel axis for edge anti-aliasing.
const SUPER: usize = 3;
const GREEN: Rgb = (0x00, 0xE6, 0x4D);

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    blend(b, a, t.clamp(0.0, 1.0))
}

/// Piecewise-linear color ramp over `stops`, which must be sorted by position.
fn ramp(stops: &[(f32, Rgb)], t: f32) -> Rgb {
    if t <= stops[0].0 {
        return stops[0].1;
    }
    for pair in stops.windows(2) {
        let (t0, c0) = pair[0];
        let (t1, c1) = pair[1];
        if t <= t1 {
            return lerp(c0, c1, (t - t0) / (t1 - t0));
        }
    }
    stops[stops.len() - 1].1
}

fn normalize(v: (f32, f32, f32)) -> (f32, f32, f32) {
    let len = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt().max(1e-6);
    (v.0 / len, v.1 / len, v.2 / len)
}

fn dot(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// Distance measure for a regular octagon: constant along its outline.
fn octagon_distance(dx: f32, dy: f32) -> f32 {
    let (ax, ay) = (dx.abs(), dy.abs());
    ax.max(ay).max((ax + ay) / std::f32::consts::SQRT_2)
}

/// Shaded ring color at an offset from the center, if the point is on the band.
fn ring_color(dx: f32, dy: f32) -> Option<Rgb> {
    let d = octagon_distance(dx, dy);
    if !(INNER..=OUTER).contains(&d) {
        return None;
    }
    // Position across the band, and the normal of a rounded tube profile.
    let t = (d - INNER) / (OUTER - INNER);
    let s = 2.0 * t - 1.0;
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let nz = (1.0 - s * s).max(0.0).sqrt();
    let n = normalize((s * dx / len * 0.9, s * dy / len * 0.9, nz));

    let light = normalize((-0.5, -0.6, 0.62));
    let diffuse = dot(n, light).max(0.0);
    let mut color = ramp(
        &[
            (0.00, (0x02, 0x36, 0x1B)),
            (0.30, (0x00, 0x8F, 0x40)),
            (0.65, (0x00, 0xE6, 0x4D)),
            (1.00, (0x9B, 0xFF, 0xC4)),
        ],
        diffuse * 1.05,
    );

    // Cool rim light from the lower right keeps the shadow side readable.
    let rim = dot(n, normalize((0.6, 0.7, 0.2))).max(0.0).powi(3);
    color = lerp(color, (0x4F, 0xD8, 0x9A), rim * 0.35);

    // Specular highlight.
    let half = normalize((light.0, light.1, light.2 + 1.0));
    let spec = dot(n, half).max(0.0).powf(26.0);
    color = lerp(color, (0xF0, 0xFF, 0xF6), spec * 0.9);

    // Engraved groove along the middle of the band.
    if (t - 0.5).abs() < 0.05 {
        color = lerp(color, (0x00, 0x14, 0x08), 0.35);
    }
    Some(color)
}

/// Soft glow around the band, strongest on the band's centerline.
fn halo_alpha(dx: f32, dy: f32) -> f32 {
    let d = octagon_distance(dx, dy);
    let from_band = d - (INNER + OUTER) / 2.0;
    let glow = 0.5 * (-(from_band / 3.0).powi(2)).exp();
    // The hollow stays faintly lit, as if the ring were holding light.
    let ambient = if d < INNER { 0.10 } else { 0.0 };
    glow + ambient
}

/// The bars above and below the ring.
fn bar_color(x: usize) -> Rgb {
    let t = x as f32 / (WIDTH - 1) as f32;
    lerp((0x4A, 0xD9, 0x7B), (0xA6, 0xF5, 0x6B), t)
}

/// One pixel of the final image. `bg` is the terminal background when known;
/// without it the glow is dropped and edges are hard so nothing looks smudged.
fn sample(x: usize, y: usize, bg: Option<Rgb>) -> Option<Rgb> {
    if y == 0 || y == HEIGHT - 1 {
        return Some(bar_color(x));
    }
    let n = (SUPER * SUPER) as f32;
    let mut sum = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut ring_sum = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut ring_hits = 0;
    let mut glow = false;
    for sy in 0..SUPER {
        for sx in 0..SUPER {
            let px = x as f32 + (sx as f32 + 0.5) / SUPER as f32 - CENTER.0;
            let py = y as f32 + (sy as f32 + 0.5) / SUPER as f32 - CENTER.1;
            let color = match (ring_color(px, py), bg) {
                (Some(color), _) => {
                    ring_hits += 1;
                    ring_sum.0 += f32::from(color.0);
                    ring_sum.1 += f32::from(color.1);
                    ring_sum.2 += f32::from(color.2);
                    color
                }
                (None, Some(bg)) => {
                    let alpha = halo_alpha(px, py);
                    glow |= alpha >= 0.02;
                    blend(GREEN, bg, alpha)
                }
                (None, None) => (0, 0, 0),
            };
            sum.0 += f32::from(color.0);
            sum.1 += f32::from(color.1);
            sum.2 += f32::from(color.2);
        }
    }
    match bg {
        Some(_) if ring_hits == 0 && !glow => None,
        Some(_) => Some(((sum.0 / n) as u8, (sum.1 / n) as u8, (sum.2 / n) as u8)),
        None if ring_hits * 2 >= SUPER * SUPER => {
            let hits = ring_hits as f32;
            Some((
                (ring_sum.0 / hits) as u8,
                (ring_sum.1 / hits) as u8,
                (ring_sum.2 / hits) as u8,
            ))
        }
        None => None,
    }
}

/// Rasterize the ring into `ROWS` rows of `WIDTH` cells.
pub(crate) fn render(bg: Option<Rgb>) -> Vec<Vec<Cell>> {
    (0..ROWS)
        .map(|row| {
            (0..WIDTH)
                .map(|col| {
                    let top = sample(col, 2 * row, bg);
                    let bottom = sample(col, 2 * row + 1, bg);
                    match (top, bottom) {
                        (None, None) => Cell { ch: ' ', fg: None, bg: None },
                        (Some(t), None) => Cell { ch: '▀', fg: Some(t), bg: None },
                        (None, Some(b)) => Cell { ch: '▄', fg: Some(b), bg: None },
                        (Some(t), Some(b)) => Cell { ch: '▀', fg: Some(t), bg: Some(b) },
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(c: Rgb) -> u32 {
        77 * u32::from(c.0) + 150 * u32::from(c.1) + 29 * u32::from(c.2)
    }

    #[test]
    fn render_has_fixed_dimensions() {
        for bg in [None, Some((12, 12, 16))] {
            let cells = render(bg);
            assert_eq!(cells.len(), ROWS);
            assert!(cells.iter().all(|row| row.len() == WIDTH));
        }
    }

    #[test]
    fn render_is_deterministic() {
        assert_eq!(render(Some((12, 12, 16))), render(Some((12, 12, 16))));
    }

    #[test]
    fn bars_frame_a_hollow_octagonal_ring() {
        let cells = render(None);
        assert!(cells[0][3].fg.is_some(), "top bar is painted");
        assert!(cells[ROWS - 1][3].fg.is_some(), "bottom bar is painted");
        assert!(cells[7][4].fg.is_some(), "left side of the band is painted");
        assert!(cells[7][23].fg.is_some(), "right side of the band is painted");
        assert_eq!(cells[7][14].ch, ' ', "center is hollow");
        assert_eq!(cells[1][0].ch, ' ', "corner outside the ring is empty");
        // Octagon: the diagonal corner of the bounding square is cut off.
        assert_eq!(cells[1][3].ch, ' ', "outer corner is chamfered");
    }

    #[test]
    fn band_is_lit_from_the_upper_left() {
        let cells = render(None);
        let lit = cells[3][6].fg.expect("upper-left band");
        let shade = cells[11][21].fg.expect("lower-right band");
        assert!(luma(lit) > luma(shade), "{lit:?} should be brighter than {shade:?}");
        assert!(shade.1 > shade.0 && shade.1 > shade.2, "shadow side stays green: {shade:?}");
    }

    #[test]
    fn band_has_a_specular_highlight() {
        let cells = render(None);
        let brightest = cells
            .iter()
            .skip(1)
            .take(ROWS - 2) // ring rows only, not the bars
            .flatten()
            .filter_map(|c| c.fg)
            .map(luma)
            .max()
            .expect("painted cells");
        // Brighter than plain emerald, so there is a highlight, not just a fill.
        assert!(brightest > luma(GREEN) + 10_000, "brightest luma {brightest}");
    }

    #[test]
    fn glow_tints_the_hollow_only_when_background_is_known() {
        assert_eq!(render(None)[7][14].ch, ' ');
        let center = render(Some((12, 12, 16)))[7][14];
        assert!(center.fg.is_some(), "glow should tint the hollow: {center:?}");
    }
}
