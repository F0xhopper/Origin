//! UI composition: lay out the panels and draw the whole frame.

pub mod detail;
pub mod help;
pub mod statusline;
pub mod theme;
pub mod tree;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::{App, Overlay};

/// Draw the entire UI for one frame.
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Calm dark background across the whole screen.
    f.render_widget(Block::default().style(theme::text()), area);

    // Status line, the horizontal tree, then the active-node detail panel.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(7),
            Constraint::Length(9),
        ])
        .split(area);

    statusline::draw(f, app, rows[0]);
    tree::draw(f, app, rows[1]);
    detail::draw(f, app, rows[2]);

    // Overlays on top.
    if app.overlay == Overlay::Help {
        help::draw(f, area);
    }
}
