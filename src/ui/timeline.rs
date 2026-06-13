//! Central reverse-timeline view: a vertical descent from the modern form at
//! the top to the oldest root at the bottom.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::model::Word;
use crate::ui::theme;
use crate::ui::word_list::scroll_offset;

/// Each stage occupies this many rows (node line + connector line).
const STEP: u16 = 2;

/// Draw the timeline for the open word (Inspect) or the selected word (preview).
pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let inspecting = app.mode == Mode::Inspect;
    let title = if inspecting {
        " Reverse Timeline "
    } else {
        " Reverse Timeline (preview) "
    };
    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(inspecting));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let word_idx = if inspecting {
        app.open_word
    } else {
        app.selected_index()
    };
    let Some(word) = word_idx.and_then(|i| app.dataset.get(i)) else {
        let hint = Paragraph::new(Line::from(Span::styled(
            "  select a word and press Enter to dive",
            theme::dim(),
        )));
        f.render_widget(hint, inner);
        app.geom.timeline = Rect::default();
        app.geom.timeline_step = 0;
        return;
    };

    let depth = word.depth();
    let visible_stages = ((inner.height / STEP).max(1)) as usize;
    let cursor = if inspecting { app.stage_cursor } else { 0 };
    let offset = scroll_offset(cursor, depth, visible_stages);

    // Record geometry for mouse hit-testing (only interactive while inspecting).
    app.geom.timeline = inner;
    app.geom.timeline_offset = offset;
    app.geom.timeline_step = if inspecting { STEP } else { 0 };

    let lines = build_lines(
        word,
        cursor,
        inspecting,
        offset,
        visible_stages,
        inner.width,
    );
    f.render_widget(Paragraph::new(lines), inner);
}

fn build_lines(
    word: &Word,
    cursor: usize,
    inspecting: bool,
    offset: usize,
    visible: usize,
    width: u16,
) -> Vec<Line<'static>> {
    let depth = word.depth();
    let mut lines: Vec<Line<'static>> = Vec::new();

    for i in offset..(offset + visible).min(depth) {
        let layer = &word.chain[i];
        let ratio = if depth <= 1 {
            0.0
        } else {
            i as f32 / (depth - 1) as f32
        };
        let color = theme::stage_color(ratio);
        let is_active = inspecting && i == cursor;

        let glyph = if i == 0 {
            "●"
        } else if i == depth - 1 {
            "◇"
        } else {
            "◆"
        };
        let marker = if is_active { "▶ " } else { "  " };
        let form_style = if is_active {
            theme::stage_active()
        } else {
            Style::default().fg(color)
        };

        // Right-aligned language · period metadata.
        let meta = format!("{} · {}", layer.language.label(), layer.period.label);
        let left = format!("{marker}{glyph} {}", layer.form);
        let pad = (width as usize)
            .saturating_sub(left.chars().count() + meta.chars().count() + 1)
            .max(1);
        lines.push(Line::from(vec![
            Span::styled(left, form_style),
            Span::raw(" ".repeat(pad)),
            Span::styled(meta, theme::dim()),
        ]));

        // Connector to the next (older) stage.
        if i < depth - 1 {
            lines.push(Line::from(Span::styled(
                "  │",
                Style::default().fg(theme::DIM),
            )));
        }
    }

    lines
}
