//! UI composition: lay out the panels and draw the whole frame.

pub mod detail;
pub mod help;
pub mod stats_panel;
pub mod statusline;
pub mod theme;
pub mod timeline;
pub mod word_list;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::{App, Overlay};

/// Draw the entire UI for one frame.
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Calm dark background across the whole screen.
    f.render_widget(Block::default().style(theme::text()), area);

    // Status line + body.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    statusline::draw(f, app, rows[0]);

    // Body: word list | timeline | right column.
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Min(30),
            Constraint::Length(34),
        ])
        .split(rows[1]);

    word_list::draw(f, app, body[0]);
    timeline::draw(f, app, body[1]);

    // Right column: stage detail (top) + discovery stats (bottom).
    let stats_height = if app.focus_stats { 12 } else { 7 };
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(stats_height)])
        .split(body[2]);
    detail::draw(f, app, right[0]);
    stats_panel::draw(f, app, right[1]);

    // Overlays on top.
    if app.overlay == Overlay::Help {
        help::draw(f, area);
    }
}
