//! Top status line: app badge, autoplay/pending indicators, and key hints.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Overlay};
use crate::ui::theme;

/// Draw the status line across the given (1-row) area.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // App badge.
    spans.push(Span::styled(
        " ORIGIN ",
        Style::default()
            .fg(theme::GOLD_BRIGHT)
            .bg(theme::ACTIVE_BG)
            .add_modifier(Modifier::BOLD),
    ));

    // Autoplay indicator.
    if app.autoplay_active() {
        spans.push(Span::styled(
            " ▶ playing ",
            Style::default().fg(theme::GOLD),
        ));
    }

    // Pending count / g chord.
    if let Some(n) = app.pending.count {
        spans.push(Span::styled(format!(" {n}"), theme::dim()));
    }
    if app.pending.g {
        spans.push(Span::styled(" g", theme::dim()));
    }

    // Right-aligned context hint.
    let hint = match app.overlay {
        Overlay::Help => "?/Esc:close",
        Overlay::None => "h:newer  l:older  gg:modern  G:root  Space:play  ?:help  q:quit",
    };

    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let pad = (area.width as usize).saturating_sub(used + hint.chars().count() + 1);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(hint, theme::dim()));
    spans.push(Span::raw(" "));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
