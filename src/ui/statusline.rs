use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Overlay};
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled(
        " origin ",
        theme::text().add_modifier(Modifier::BOLD),
    ));

    if app.autoplay_active() {
        spans.push(Span::styled(" playing", theme::dim()));
    }

    if let Some(n) = app.pending.count {
        spans.push(Span::styled(format!(" {n}"), theme::dim()));
    }
    if app.pending.g {
        spans.push(Span::styled(" g", theme::dim()));
    }

    let hint = match app.overlay {
        Overlay::Help => "?/Esc:close",
        Overlay::None => "?:help  q:quit",
    };

    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let pad = (area.width as usize).saturating_sub(used + hint.chars().count() + 1);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(hint, theme::dim()));
    spans.push(Span::raw(" "));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
