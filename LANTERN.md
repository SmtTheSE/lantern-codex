# Lantern Codex

A fan-made, Green Lantern flavored fork of the [OpenAI Codex CLI](https://github.com/openai/codex) (Apache-2.0).

> **Not affiliated with, endorsed by, or sponsored by DC, Warner Bros. Discovery, HBO, or OpenAI.**
> Green Lantern, Lanterns and related names, logos and the oath are trademarks and/or copyrighted works of their owners.

## What changes

| Stock Codex | Lantern Codex |
| --- | --- |
| `Working` while a turn runs | A phrase from [`lantern.rs`](codex-rs/tui/src/lantern.rs) chosen to fit what the model is doing (searching, editing, testing or planning), with an ellipsis and rotating every 7s. The model's own status text stays visible after the timer. |
| Nothing under the working row | One line of the oath, rotating with the phrase |
| Plain footer status line | Green status line: `[model effort] │ dir │ 5h ━━━━━━── 80% │ 7d ━━━━━━━─ 93% │ ctx ━━━━━━── 25%`. Bars turn yellow below 25% remaining. |
| White shimmer | Ring-green shimmer |
| `OpenAI Codex` banner | Rendered emerald ring between two bars, `LANTERN CODEX · Sector 2814`, the oath, directory and model |

The status line only fills in the limit and context bars once Codex has that data (after a turn). If you set `tui.status_line` in `config.toml`, your items are used instead of the lantern defaults, but they are still drawn in green.

### Construct badge

On truecolor terminals the working row starts with an animated construct: eight cells assemble one by one through `░▒▓█` with a white-green flash as each piece locks in, a sparkle sweeps the finished construct, then the pieces dissolve right to left and it starts over (3.6s cycle). It lives in [`lantern_construct.rs`](codex-rs/tui/src/lantern_construct.rs) as a pure function of elapsed time. With animations disabled or without truecolor you get the stock spinner.

### Everything green

Cyan, blue and magenta (ANSI and RGB) are remapped to Lantern greens at the three places colors reach the terminal (`recolor` in `lantern.rs`), so the picker highlights, links, mode labels and transcript accents all follow the theme, including anything added upstream later. Red and yellow are left alone, so errors and warnings still read as errors and warnings.

## Themes

Set `CODEX_LANTERN_THEME` to pick the palette:

- `heartland` (default), inspired by the series' look (dusty Nebraska earth tones, green used sparingly): bone and straw text, umber rules, ring green kept for the accent, and a gold glint when a construct locks in.
- `corps`: everything ring green.

## The oath

`OATH` in `lantern.rs` holds the four lines used in the banner and under the working row. They are DC's text, included at the fork author's choice and not covered by this project's license. If DC or Warner Bros. asks, or if you simply prefer, replace them with your own wording: the only requirement is four lines, and the last one is highlighted in ring green.

## Turning it off

```bash
CODEX_LANTERN=0 codex
```

## Adding phrases

Edit the pools (`INVESTIGATE`, `BUILD`, `VERIFY`, `THINK`, `GENERAL`) in `codex-rs/tui/src/lantern.rs`. Keep each line to 40 characters or fewer and free of " by <character>" attributions (unit tests enforce both). Phrases are original riffs on the series' premise, not quotes from the show, and are shown with a trailing ellipsis.

## Logo

The startup banner shows an octagonal emerald ring between two bars, like the reference terminal: `LANTERN CODEX · Sector 2814`, a glow rule, the oath, directory and model beside it. The ring is rendered in code by [`lantern_logo.rs`](codex-rs/tui/src/lantern_logo.rs) rather than drawn by hand: a bevelled tube lit from the upper left (diffuse shading, specular highlight, rim light, engraved groove), supersampled for smooth edges, with a soft glow over your terminal background.

It needs a truecolor terminal at least 60 columns wide; otherwise you get the stock banner. It is an original generic ring, not the Corps emblem, and no official artwork ships in this repo.

## Build

```bash
cd codex-rs
cargo build -p codex-cli --release
./target/release/codex
```

Upstream Codex is licensed under Apache-2.0; see `LICENSE` and `NOTICE`.
