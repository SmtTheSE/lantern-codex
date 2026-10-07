//! Lantern theme: rotating working phrases, the oath, colors and brand title.
//!
//! Two looks, chosen with `CODEX_LANTERN_THEME`:
//! - `heartland` (default), after the 2026 series: dusty earth tones (straw,
//!   bone, umber) for text, with ring green kept as the one accent and a gold
//!   glint when a construct locks in.
//! - `corps`: everything ring green.
//!
//! Set `CODEX_LANTERN=0` to restore stock Codex text and colors. Unit and
//! snapshot tests always see the stock text so upstream tests stay valid.
//!
//! Not affiliated with or endorsed by DC, Warner Bros. or HBO. The oath below
//! is DC's text, included at the fork author's choice. Replace `OATH` with your
//! own lines if the rights holder asks, or if you prefer.

use ratatui::style::Color;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Ring green used for the shimmer, the banner and the status line.
pub(crate) const RING_GREEN: (u8, u8, u8) = (0x00, 0xE6, 0x4D);

/// Muted green for secondary status text.
pub(crate) const DIM_GREEN: (u8, u8, u8) = (0x4F, 0xA8, 0x6E);

/// Heartland palette: dusty earth tones from the series' Nebraska look.
/// Bone is primary text, straw is warm emphasis, dust is secondary text and
/// umber is rules and empty bar cells.
pub(crate) const BONE: (u8, u8, u8) = (0xE8, 0xE0, 0xD0);
pub(crate) const STRAW: (u8, u8, u8) = (0xC9, 0xA9, 0x6E);
pub(crate) const DUST: (u8, u8, u8) = (0x8A, 0x7F, 0x6E);
pub(crate) const UMBER: (u8, u8, u8) = (0x4A, 0x3B, 0x2C);

/// The ring is reported as gold with a green head; this is the gold.
pub(crate) const RING_GOLD: (u8, u8, u8) = (0xF2, 0xD2, 0x7A);

/// Warning color for low limits: the one color the ring fears.
pub(crate) const YELLOW_IMPURITY: (u8, u8, u8) = (0xF5, 0xD0, 0x20);

/// The oath, one line per row. Shown in the banner and under the working row.
pub(crate) const OATH: [&str; 4] = [
    "In brightest day, in blackest night,",
    "No evil shall escape my sight.",
    "Let those who worship evil's might,",
    "Beware my power, Green Lantern's light!",
];

/// Status line used when the user has not configured one: model, directory,
/// rate-limit bars and context, like the reference terminal.
pub(crate) const DEFAULT_STATUS_ITEMS: [&str; 5] = [
    "model-with-reasoning",
    "current-dir",
    "five-hour-limit",
    "weekly-limit",
    "context-used",
];

/// What the model is doing, inferred from its own status text, so the working
/// label can fit the activity. All labels are original lines that riff on the
/// series' premise and motifs (a murder in Rushville, Nebraska; diners and a
/// sheriff; a rookie still training; constructs made of willpower). None are
/// quotes from the show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkKind {
    /// Reading, searching, mapping the codebase.
    Investigate,
    /// Writing and editing code.
    Build,
    /// Running tests, builds, linters.
    Verify,
    /// Planning and reasoning.
    Think,
    /// No specific activity known.
    General,
}

/// Classify the model's status text (for example "Running tests").
pub(crate) fn classify(header: &str) -> WorkKind {
    let h = header.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| h.contains(w));
    if has(&["test", "lint", "compil", "verif", "validat", "format", "typecheck", "running", "execut"]) {
        WorkKind::Verify
    } else if has(&["think", "plan", "reason", "consider", "decid", "draft", "design", "weigh", "analy"]) {
        // Before Build, so "Planning the refactor" reads as planning.
        WorkKind::Think
    } else if has(&[
        "edit", "writ", "updat", "patch", "apply", "implement", "creat", "add", "refactor", "fix",
        "chang", "modif", "build",
    ]) {
        WorkKind::Build
    } else if has(&[
        "search", "read", "look", "find", "explor", "map", "scan", "inspect", "review", "investigat",
        "list", "grep", "open", "locat", "trac", "understand", "check",
    ]) {
        WorkKind::Investigate
    } else {
        WorkKind::General
    }
}

const INVESTIGATE: &[&str] = &[
    "Working the case in Rushville",
    "Chasing the hail in Rushville",
    "Canvassing the diner",
    "Reading the scene for green",
    "Checking the sheriff's files",
    "Tracing the signal across Nebraska",
    "Interviewing a witness (the alien kind)",
    "Following the dusty highway",
    "Searching for green in this scene",
    "Scanning sector 2814",
];

const BUILD: &[&str] = &[
    "Shaping",
    "Constructing",
    "Willpowering",
    "Believing it into existence",
    "Welding the ring-light seams",
    "Imagination, with a style guide",
    "Rookie hands, steady ring",
    "Building it like a Lantern would",
    "Constructing something sturdy",
    "Shaping a construct",
];

const VERIFY: &[&str] = &[
    "Running drills before the ring",
    "Dusting for prints",
    "Cross-examining the build",
    "Bail hearing for a flaky test",
    "Checking the evidence twice",
    "Training, ring not included",
    "Yellow impurity detected, retrying",
    "Filing a report with the Guardians",
    "Testing the construct's weight",
    "No ring needed for this check",
];

const THINK: &[&str] = &[
    "Trusting my gut",
    "Believing",
    "Focusing",
    "Sitting in the diner, thinking",
    "Waiting for the dust to settle",
    "Reading the horizon",
    "Allowed to be a little afraid",
    "Overthinking, but with a power ring",
    "Reciting the oath",
    "Thoughts on this build?",
];

const GENERAL: &[&str] = &[
    "Shaping",
    "Willpowering",
    "Constructing",
    "Focusing",
    "Believing",
    "Charging the ring",
    "Recharging at the lantern",
    "Driving through Nebraska",
    "Heading for the diner",
    "True Detective, but with power rings",
    "Reciting the oath",
    "Scanning sector 2814",
];

fn pool(kind: WorkKind) -> &'static [&'static str] {
    match kind {
        WorkKind::Investigate => INVESTIGATE,
        WorkKind::Build => BUILD,
        WorkKind::Verify => VERIFY,
        WorkKind::Think => THINK,
        WorkKind::General => GENERAL,
    }
}

/// Seconds each phrase and oath line stays on screen.
const ROTATE_SECONDS: u64 = 7;

static NEXT_BASE: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn enabled() -> bool {
    if cfg!(test) {
        return false;
    }
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        !matches!(
            std::env::var("CODEX_LANTERN").as_deref(),
            Ok("0" | "false" | "off")
        )
    })
}

/// Whether the dusty heartland look is active (the default theme).
pub(crate) fn heartland() -> bool {
    if !enabled() {
        return false;
    }
    static HEARTLAND: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *HEARTLAND.get_or_init(|| {
        !matches!(
            std::env::var("CODEX_LANTERN_THEME").as_deref(),
            Ok("corps" | "green")
        )
    })
}

/// Map cool UI colors (cyan, blue, magenta, ANSI or RGB) onto the theme so the
/// whole interface stays on-palette, wherever a color originates. Red and
/// yellow are left alone so errors and warnings still read as such.
pub(crate) fn recolor(color: Color) -> Color {
    if enabled() { lantern_color(color, heartland()) } else { color }
}

fn lantern_color(color: Color, heartland: bool) -> Color {
    let rgb = |(r, g, b): (u8, u8, u8)| Color::Rgb(r, g, b);
    match (color, heartland) {
        (Color::Cyan, _) => rgb(RING_GREEN),
        (Color::LightCyan, _) => rgb((0x7D, 0xFF, 0xB0)),
        (Color::Blue, true) => rgb((0x3F, 0xB8, 0x6B)),
        (Color::Blue, false) => rgb((0x1F, 0xA8, 0x55)),
        (Color::LightBlue, true) => rgb((0x7B, 0xD8, 0x9A)),
        (Color::LightBlue, false) => rgb((0x6B, 0xF2, 0xA0)),
        // Heartland keeps magenta warm (straw); the all-green theme goes green.
        (Color::Magenta, true) => rgb(STRAW),
        (Color::Magenta, false) => rgb((0x39, 0xD3, 0x6B)),
        (Color::LightMagenta, true) => rgb((0xE3, 0xCB, 0x98)),
        (Color::LightMagenta, false) => rgb((0x9B, 0xFF, 0xC4)),
        // Blue- or cyan-leaning RGB keeps its brightness but turns green.
        (Color::Rgb(r, g, b), _) if b > r.saturating_add(30) && b.saturating_add(10) >= g => {
            rgb((r / 3, g.max(b), b / 3))
        }
        (other, _) => other,
    }
}

/// Starting offset for a new working row, so turns and sessions do not all
/// open on the same phrase.
pub(crate) fn next_base() -> usize {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos() as usize);
    NEXT_BASE.fetch_add(1, Ordering::Relaxed) + seed
}

/// The working phrase to show `elapsed_secs` into a turn, fitted to what the
/// model is doing (`header` is its own status text, or "Working").
pub(crate) fn phrase(base: usize, elapsed_secs: u64, header: &str) -> &'static str {
    let pool = pool(classify(header));
    pool[(base + (elapsed_secs / ROTATE_SECONDS) as usize) % pool.len()]
}

/// The oath line to show under the working row `elapsed_secs` into a turn.
pub(crate) fn oath_line(base: usize, elapsed_secs: u64) -> &'static str {
    OATH[(base + (elapsed_secs / ROTATE_SECONDS) as usize) % OATH.len()]
}

/// Foreground used for the shimmer's bright band.
pub(crate) fn shimmer_fg(stock: (u8, u8, u8)) -> (u8, u8, u8) {
    if enabled() { RING_GREEN } else { stock }
}

/// Brand name shown in the session header and status card.
pub(crate) fn title() -> &'static str {
    if enabled() {
        "Lantern Codex"
    } else {
        "OpenAI Codex"
    }
}

/// Terminal width needed to put the ring image beside the header text.
const RING_MIN_TERMINAL_WIDTH: usize = 60;

fn truecolor() -> bool {
    matches!(
        crate::terminal_palette::effective_stdout_color_level(),
        crate::terminal_palette::StdoutColorLevel::TrueColor
    )
}

/// Whether to draw the ring beside the header text: theme on, truecolor
/// available and a terminal wide enough to fit it.
pub(crate) fn show_logo(terminal_width: usize) -> bool {
    enabled() && terminal_width >= RING_MIN_TERMINAL_WIDTH && truecolor()
}

/// Whether the working row should show the animated construct badge instead
/// of the stock spinner. Needs truecolor for the flash and fade gradients.
pub(crate) fn show_construct() -> bool {
    enabled() && truecolor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_text_under_test() {
        assert!(!enabled());
        assert_eq!(title(), "OpenAI Codex");
    }

    #[test]
    fn recolor_is_identity_under_test() {
        assert_eq!(recolor(Color::Cyan), Color::Cyan);
        assert_eq!(recolor(Color::Rgb(99, 168, 248)), Color::Rgb(99, 168, 248));
    }

    #[test]
    fn cool_colors_follow_the_theme_and_signals_are_kept() {
        let is_green = |c: Color| matches!(c, Color::Rgb(r, g, b) if g > r && g > b);
        for heartland in [true, false] {
            for cool in [
                Color::Cyan,
                Color::LightCyan,
                Color::Blue,
                Color::LightBlue,
                Color::Rgb(99, 168, 248),
                Color::Rgb(28, 100, 200),
                Color::Rgb(0, 255, 255),
            ] {
                let mapped = lantern_color(cool, heartland);
                assert!(is_green(mapped), "{cool:?} -> {mapped:?} (heartland={heartland})");
            }
            for kept in [Color::Red, Color::Yellow, Color::Green, Color::Reset, Color::Rgb(200, 60, 60)] {
                assert_eq!(lantern_color(kept, heartland), kept);
            }
        }
    }

    #[test]
    fn magenta_is_warm_in_heartland_and_green_in_corps() {
        let (r, g, b) = STRAW;
        assert_eq!(lantern_color(Color::Magenta, true), Color::Rgb(r, g, b));
        assert!(matches!(lantern_color(Color::Magenta, false), Color::Rgb(r, g, b) if g > r && g > b));
    }

    #[test]
    fn every_pool_is_unique_short_and_unattributed() {
        for kind in [
            WorkKind::Investigate,
            WorkKind::Build,
            WorkKind::Verify,
            WorkKind::Think,
            WorkKind::General,
        ] {
            let mut seen = std::collections::HashSet::new();
            for phrase in pool(kind) {
                assert!(seen.insert(*phrase), "duplicate in {kind:?}: {phrase}");
                assert!(phrase.len() <= 40, "too long for narrow terminals: {phrase}");
                assert!(!phrase.contains(" by "), "no character attributions: {phrase}");
            }
        }
    }

    #[test]
    fn status_text_picks_the_matching_pool() {
        assert_eq!(classify("Running tests"), WorkKind::Verify);
        assert_eq!(classify("Editing config.rs"), WorkKind::Build);
        assert_eq!(classify("Mapping the app structure"), WorkKind::Investigate);
        assert_eq!(classify("Planning the refactor"), WorkKind::Think);
        assert_eq!(classify("Working"), WorkKind::General);
    }

    #[test]
    fn phrase_rotates_every_interval_and_wraps() {
        let pool = pool(WorkKind::General);
        assert_eq!(phrase(0, 0, "Working"), pool[0]);
        assert_eq!(phrase(0, ROTATE_SECONDS - 1, "Working"), pool[0]);
        assert_eq!(phrase(0, ROTATE_SECONDS, "Working"), pool[1]);
        assert_eq!(phrase(pool.len() - 1, ROTATE_SECONDS, "Working"), pool[0]);
        assert_eq!(oath_line(OATH.len() - 1, ROTATE_SECONDS), OATH[0]);
    }
}
