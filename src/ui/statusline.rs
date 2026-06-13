//! Top status line: mode indicator, pending chord/count, search, key hints.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Mode, Overlay};
use crate::ui::theme;

/// Draw the status line across the given (1-row) area.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    // Mode badge.
    spans.push(Span::styled(
        format!(" {} ", app.mode.label()),
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

    // Active search query.
    if app.overlay == Overlay::Search {
        spans.push(Span::styled(
            format!("  /{}", app.search_query),
            Style::default().fg(theme::GOLD_BRIGHT),
        ));
        spans.push(Span::styled("█", Style::default().fg(theme::GOLD_BRIGHT)));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(
            format!("  filter:/{}", app.search_query),
            theme::dim(),
        ));
    }

    // Right-aligned context hint.
    let hint = match (app.overlay, app.mode) {
        (Overlay::Search, _) => "type to filter  Enter:keep  Esc:cancel",
        (Overlay::Help, _) => "?/Esc:close",
        (Overlay::None, Mode::Navigation) => {
            "j/k:move  Enter:dive  /:search  r:random  ?:help  q:quit"
        }
        (Overlay::None, Mode::Inspect) => {
            "h:older  l:newer  gg:root  G:modern  Space:play  Esc:back"
        }
    };

    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let pad = (area.width as usize).saturating_sub(used + hint.chars().count() + 1);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(hint, theme::dim()));
    spans.push(Span::raw(" "));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
