//! Left navigation panel: the scrollable list of words.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::ui::theme;

/// Keep the cursor visible: first visible index for a viewport.
pub fn scroll_offset(cursor: usize, len: usize, height: usize) -> usize {
    if height == 0 || len <= height {
        return 0;
    }
    let max_off = len - height;
    cursor.saturating_sub(height / 2).min(max_off)
}

/// Draw the word list and record its geometry for mouse hit-testing.
pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.mode == Mode::Navigation;
    let block = Block::default()
        .title(Span::styled(" Words ", theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let len = app.filtered.len();
    let offset = scroll_offset(app.list_cursor, len, height);

    // Record geometry for mouse clicks.
    app.geom.word_list = inner;
    app.geom.word_list_offset = offset;

    if len == 0 {
        let p = Paragraph::new(Line::from(Span::styled("  no matches", theme::dim())));
        f.render_widget(p, inner);
        return;
    }

    let mut lines = Vec::with_capacity(height);
    for (row, &word_idx) in app.filtered.iter().enumerate().skip(offset).take(height) {
        let word = match app.dataset.get(word_idx) {
            Some(w) => w,
            None => continue,
        };
        let is_sel = row == app.list_cursor;
        let marker = if is_sel { "▶ " } else { "  " };
        let style = if is_sel {
            theme::selected()
        } else {
            theme::text()
        };
        let label = format!("{marker}{}", word.headword);
        lines.push(Line::from(Span::styled(label, style)));
    }

    f.render_widget(Paragraph::new(lines), inner);
}
