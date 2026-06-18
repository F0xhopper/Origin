//! Centered help overlay listing the Vim-style keybindings.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme;

const ROWS: &[(&str, &str)] = &[
    ("l / →", "step back in time (older form)"),
    ("h / ←", "step forward (newer form)"),
    ("gg / 0", "jump to the modern word"),
    ("G / $", "jump to the oldest root"),
    ("Space", "play / pause backward auto-traversal"),
    ("3l", "numeric counts repeat a motion"),
    ("mouse", "click a node, scroll to navigate"),
    ("?", "toggle this help"),
    ("q / Esc", "quit"),
];

/// Draw the help overlay centered on screen.
pub fn draw(f: &mut Frame, area: Rect) {
    let popup = centered(64, (ROWS.len() + 4) as u16, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(" Help — Vim controls ", theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(true));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines: Vec<Line> = ROWS
        .iter()
        .map(|(keys, desc)| {
            Line::from(vec![
                Span::styled(format!("  {keys:<10}"), theme::title()),
                Span::styled((*desc).to_string(), theme::text()),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

/// Compute a centered rect of the given size, clamped to `area`.
fn centered(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vert[1]);
    horiz[1]
}
