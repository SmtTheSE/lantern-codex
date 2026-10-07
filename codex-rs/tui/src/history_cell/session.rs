//! Session headers, onboarding guidance, and transcript cards.

use super::*;
use crate::line_truncation::line_width;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use crate::style::accent_color;
use crate::width::display_width;

/// Render `lines` inside a border whose inner width is at least `inner_width`.
///
/// This is useful when callers have already clamped their content to a
/// specific width and want the border math centralized here instead of
/// duplicating padding logic in the TUI widgets themselves.
pub(crate) fn with_border_with_inner_width(
    lines: Vec<Line<'static>>,
    inner_width: usize,
) -> Vec<Line<'static>> {
    let max_line_width = lines.iter().map(line_width).max().unwrap_or(0);
    let content_width = inner_width.max(max_line_width);

    let mut out = Vec::with_capacity(lines.len() + 2);
    let border_inner_width = content_width + 2;
    out.push(vec![format!("╭{}╮", "─".repeat(border_inner_width)).dim()].into());

    for line in lines.into_iter() {
        let used_width = line_width(&line);
        let span_count = line.spans.len();
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(span_count + 4);
        spans.push(Span::from("│ ").dim());
        spans.extend(line);
        if used_width < content_width {
            spans.push(Span::from(" ".repeat(content_width - used_width)).dim());
        }
        spans.push(Span::from(" │").dim());
        out.push(Line::from(spans));
    }

    out.push(vec![format!("╰{}╯", "─".repeat(border_inner_width)).dim()].into());

    out
}

/// Brand title shared by the session header and the status card; each owns its own indentation.
pub(crate) fn codex_title(version: &str) -> Vec<Span<'static>> {
    vec![
        ">_ ".fg(accent_color()),
        crate::lantern::title().bold(),
        format!(" (v{version})").dim(),
    ]
}

#[derive(Debug)]
struct TooltipHistoryCell {
    tip: String,
    cwd: PathBuf,
}

impl TooltipHistoryCell {
    fn new(tip: String, cwd: &Path) -> Self {
        Self {
            tip,
            cwd: cwd.to_path_buf(),
        }
    }
}

impl HistoryCell for TooltipHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.display_hyperlink_lines(width))
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let indent = "  ";
        let indent_width = display_width(indent);
        let wrap_width = usize::from(width.max(1))
            .saturating_sub(indent_width)
            .max(1);
        let lines = crate::tooltips::render_tooltip_lines(&self.tip, wrap_width, &self.cwd);

        prefix_hyperlink_lines(lines, indent.into(), indent.into())
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        vec![Line::from(format!("Tip: {}", self.tip))]
    }
}

/// Startup metadata, including prior-session summaries and available usage resets.
#[derive(Debug)]
pub(crate) struct SessionNoticeCell(pub(crate) PlainHistoryCell);

impl HistoryCell for SessionNoticeCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.0.display_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.0.raw_lines()
    }
}

#[derive(Debug)]
pub struct SessionInfoCell(CompositeHistoryCell);

/// Fullscreen transcript presentation omits tips; scrollback retains the original cells.
pub(crate) fn fullscreen_session_lines(
    cell: &dyn HistoryCell,
    width: u16,
    detailed: bool,
    mode: HistoryRenderMode,
) -> Vec<HyperlinkLine> {
    if let Some(info) = cell.as_any().downcast_ref::<SessionInfoCell>() {
        let mut lines = Vec::new();
        for part in &info.0.parts {
            if part.as_any().is::<TooltipHistoryCell>() {
                continue;
            }
            let part_lines = fullscreen_session_lines(part.as_ref(), width, detailed, mode);
            if !part_lines.is_empty() {
                if !lines.is_empty() {
                    lines.push(HyperlinkLine::from(""));
                }
                lines.extend(part_lines);
            }
        }
        lines
    } else if detailed || mode == HistoryRenderMode::Rich {
        cell.retained_hyperlink_lines(width, detailed)
    } else {
        cell.display_hyperlink_lines_for_mode(width, mode)
    }
}

impl HistoryCell for SessionInfoCell {
    fn compact_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.0.compact_hyperlink_lines(width)
    }

    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.0.display_lines(width)
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.0.display_hyperlink_lines(width)
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.0.desired_height(width)
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.0.transcript_lines(width)
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.0.transcript_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.0.raw_lines()
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "keep local preferences separate while the legacy Config parameter is still required"
)]
pub(crate) fn new_session_info(
    config: &Config,
    local_settings: &crate::local_settings::LocalSettings,
    requested_model: &str,
    model_display_name: &str,
    session: &ThreadSessionState,
    is_first_event: bool,
    tooltip_override: Option<String>,
    auth_plan: Option<PlanType>,
) -> SessionInfoCell {
    // Header rendered as history (so it appears at the very top).
    let header = SessionHeaderHistoryCell::new(
        model_display_name.to_string(),
        session.reasoning_effort.clone(),
        config.cwd.to_path_buf(),
        CODEX_CLI_VERSION,
    )
    .with_yolo_mode(has_yolo_permissions(
        session.approval_policy,
        &session.permission_profile,
    ));
    let mut parts: Vec<Box<dyn HistoryCell>> = vec![Box::new(header)];

    if is_first_event {
        // Help lines below the header (new copy and list)
        let help_lines: Vec<Line<'static>> = vec![
            "  To get started, describe a task or try one of these commands:"
                .dim()
                .into(),
            Line::from(""),
            Line::from(vec![
                "  ".into(),
                "/init".into(),
                " - create an AGENTS.md file with instructions for Codex".dim(),
            ]),
            Line::from(vec![
                "  ".into(),
                "/status".into(),
                " - show current session configuration".dim(),
            ]),
            Line::from(vec![
                "  ".into(),
                "/permissions".into(),
                " - choose what Codex is allowed to do".dim(),
            ]),
            Line::from(vec![
                "  ".into(),
                "/model".into(),
                " - choose what model and reasoning effort to use".dim(),
            ]),
            Line::from(vec![
                "  ".into(),
                "/review".into(),
                " - review any changes and find issues".dim(),
            ]),
        ];

        parts.push(Box::new(PlainHistoryCell { lines: help_lines }));
    } else {
        if local_settings.tui.show_tooltips
            && let Some(tooltips) = tooltip_override
                .or_else(|| tooltips::get_tooltip(auth_plan, &local_settings.tui.keymap))
                .map(|tip| TooltipHistoryCell::new(tip, &config.cwd))
        {
            parts.push(Box::new(tooltips));
        }
        if requested_model != session.model.as_str() {
            let lines = vec![
                "model changed:".magenta().bold().into(),
                format!("requested: {requested_model}").into(),
                format!("used: {}", session.model).into(),
            ];
            parts.push(Box::new(PlainHistoryCell { lines }));
        }
    }

    SessionInfoCell(CompositeHistoryCell { parts })
}

pub(crate) fn is_yolo_mode(config: &Config) -> bool {
    has_yolo_permissions(
        AskForApproval::from(config.permissions.approval_policy.value()),
        &config.permissions.effective_permission_profile(),
    )
}

pub(crate) fn has_yolo_permissions(
    approval_policy: AskForApproval,
    permission_profile: &PermissionProfile,
) -> bool {
    approval_policy == AskForApproval::Never
        && matches!(
            permission_profile,
            PermissionProfile::Disabled
                | PermissionProfile::Managed {
                    file_system: ManagedFileSystemPermissions::Unrestricted,
                    network: NetworkSandboxPolicy::Enabled,
                }
        )
}
/// Session banner with a model label already resolved for presentation by its caller.
#[derive(Debug)]
pub(crate) struct SessionHeaderHistoryCell {
    version: &'static str,
    model: String,
    reasoning_effort: Option<ReasoningEffortConfig>,
    directory: PathBuf,
    yolo_mode: bool,
}

impl SessionHeaderHistoryCell {
    pub(crate) fn new(
        model: String,
        reasoning_effort: Option<ReasoningEffortConfig>,
        directory: PathBuf,
        version: &'static str,
    ) -> Self {
        Self {
            version,
            model,
            reasoning_effort,
            directory,
            yolo_mode: false,
        }
    }

    pub(crate) fn with_yolo_mode(mut self, yolo_mode: bool) -> Self {
        self.yolo_mode = yolo_mode;
        self
    }

    fn format_directory(&self, max_width: Option<usize>) -> String {
        Self::format_directory_inner(&self.directory, max_width)
    }

    pub(crate) fn format_directory_inner(directory: &Path, max_width: Option<usize>) -> String {
        let formatted = if let Some(rel) = relativize_to_home(directory) {
            if rel.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display())
            }
        } else {
            directory.display().to_string()
        };

        if let Some(max_width) = max_width {
            if max_width == 0 {
                return String::new();
            }
            if display_width(formatted.as_str()) > max_width {
                return crate::text_formatting::center_truncate_path(&formatted, max_width);
            }
        }

        formatted
    }

    /// Themed banner: the ring on the left; directory, wordmark with sector
    /// readout, glow rule, tagline, oath and model on the right.
    fn banner_lines(
        &self,
        width: usize,
        image: Vec<Vec<crate::lantern_logo::Cell>>,
    ) -> Vec<Line<'static>> {
        use ratatui::style::Style;

        let rgb = crate::terminal_palette::rgb_color;
        let heartland = crate::lantern::heartland();
        let green = rgb(crate::lantern::RING_GREEN);
        let bone = rgb(crate::lantern::BONE);
        let straw = rgb(crate::lantern::STRAW);
        let dust = rgb(crate::lantern::DUST);
        let image_width = image.first().map_or(0, Vec::len);
        let text_width = width.saturating_sub(image_width + 5);

        // Wordmark: letter-spaced capitals like the series' title card, then
        // the sector readout from the reference terminal.
        let mut wordmark: Vec<Span<'static>> = if heartland {
            let spaced: String = "LANTERN  CODEX"
                .chars()
                .flat_map(|c| [c, ' '])
                .collect::<String>()
                .trim_end()
                .to_string();
            vec![Span::styled(spaced, Style::default().fg(bone).bold())]
        } else {
            let name = crate::lantern::title().to_uppercase();
            let letters = name.chars().count().max(2) - 1;
            name.chars()
                .enumerate()
                .map(|(i, ch)| {
                    let t = i as f32 / letters as f32;
                    let color = crate::color::blend((0xB8, 0xFF, 0xD0), (0x00, 0xC8, 0x53), t);
                    Span::styled(ch.to_string(), Style::default().fg(rgb(color)).bold())
                })
                .collect()
        };
        wordmark.push(Span::styled(" · ", Style::default().fg(dust)));
        wordmark.push(Span::styled("Sector 2814", Style::default().fg(green)));
        wordmark.push(format!("  v{}", self.version).dim());

        let oath_style = |i: usize| {
            let color = if heartland && i + 1 < crate::lantern::OATH.len() { dust } else { green };
            Style::default().fg(color).italic()
        };
        let model_color = if heartland { straw } else { green };
        let mut model = vec![Span::styled(self.model.clone(), Style::default().fg(model_color))];
        if let Some(effort) = self.reasoning_label() {
            model.push(format!(" · {effort}").dim());
        }
        if self.yolo_mode {
            model.push("  YOLO mode".magenta().bold());
        }

        // Emerald glow rule under the wordmark, fading outward.
        let rule_len = text_width.min(30);
        let rule: Vec<Span<'static>> = (0..rule_len)
            .map(|i| {
                let t = i as f32 / rule_len.max(1) as f32;
                let color = crate::color::blend(
                    crate::lantern::RING_GREEN,
                    crate::lantern::UMBER,
                    1.0 - t,
                );
                Span::styled("━", Style::default().fg(rgb(color)))
            })
            .collect();

        let mut beside: Vec<Vec<Span<'static>>> = vec![Vec::new(); image.len()];
        // Text is centered against the taller ring.
        let top = image.len().saturating_sub(11) / 2;
        beside[top + 1] = vec![self.format_directory(Some(text_width)).dim()];
        beside[top + 3] = wordmark;
        beside[top + 4] = rule;
        beside[top + 5] = vec![Span::styled(
            "Power rings. Small town. Open source.",
            Style::default().fg(straw).italic(),
        )];
        for (i, line) in crate::lantern::OATH.iter().enumerate() {
            beside[top + 6 + i] = vec![Span::styled(*line, oath_style(i))];
        }
        beside[top + 10] = model;

        let mut lines = vec![Line::default()];
        for (cells, text) in image.into_iter().zip(beside) {
            let mut spans: Vec<Span<'static>> = vec![Span::from("  ")];
            for crate::lantern_logo::Cell { ch, fg, bg } in cells {
                let mut style = Style::default();
                if let Some(fg) = fg {
                    style = style.fg(rgb(fg));
                }
                if let Some(bg) = bg {
                    style = style.bg(rgb(bg));
                }
                spans.push(Span::styled(ch.to_string(), style));
            }
            if !text.is_empty() {
                spans.push(Span::from("   "));
                spans.extend(text);
            }
            lines.push(truncate_line_with_ellipsis_if_overflow(Line::from(spans), width));
        }
        lines
    }

    fn reasoning_label(&self) -> Option<&str> {
        self.reasoning_effort
            .as_ref()
            .map(ReasoningEffortConfig::as_str)
    }
}

impl HistoryCell for SessionHeaderHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let width = usize::from(width);
        if crate::lantern::show_logo(width) {
            let image = crate::lantern_logo::render(crate::terminal_palette::default_bg());
            return self.banner_lines(width, image);
        }
        let mut title = vec!["  ".into()];
        title.extend(codex_title(self.version));
        let mut lines = vec![
            Line::default(),
            Line::from(title),
            Line::from(vec![
                "     ".into(),
                self.format_directory(Some(width.saturating_sub(/*rhs*/ 5)))
                    .dim(),
            ]),
        ];
        if self.yolo_mode {
            lines.push(Line::from(vec![
                "  permissions: ".dim(),
                "YOLO mode".magenta().bold(),
            ]));
        }
        lines
            .into_iter()
            .map(|line| truncate_line_with_ellipsis_if_overflow(line, width))
            .collect()
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(format!("{} (v{})", crate::lantern::title(), self.version)),
            Line::from(format!(
                "model: {}{}",
                self.model,
                self.reasoning_label()
                    .map(|reasoning| format!(" {reasoning}"))
                    .unwrap_or_default()
            )),
            Line::from(format!(
                "directory: {}",
                self.format_directory(/*max_width*/ None)
            )),
        ];
        if self.yolo_mode {
            lines.push(Line::from("permissions: YOLO mode"));
        }
        lines
    }
}

#[cfg(test)]
#[path = "session_transcript_tests.rs"]
mod transcript_tests;
