//! The horizontal etymology tree: a left-to-right chain of nodes from the
//! modern form to the oldest root, with the active node highlighted.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::model::Word;
use crate::ui::theme;

const CONNECTOR: &str = " ──▶ ";
/// The tree occupies this many text rows (form, language, period).
const TREE_ROWS: u16 = 3;

/// Keep the cursor visible: index of the first node to show in a viewport.
pub fn scroll_offset(cursor: usize, len: usize, visible: usize) -> usize {
    if visible == 0 || len <= visible {
        return 0;
    }
    let max_off = len - visible;
    cursor.saturating_sub(visible / 2).min(max_off)
}

/// Column width for each node, sized to the longest form in the chain.
fn node_width(word: &Word) -> usize {
    word.chain
        .iter()
        .map(|l| l.form.chars().count())
        .max()
        .unwrap_or(6)
        .clamp(6, 22)
}

/// Draw the tree for the open word and record geometry for mouse hit-testing.
pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let word = &app.word;
    let depth = word.depth();
    let node_w = node_width(word);
    let step = node_w + CONNECTOR.chars().count();

    let visible = ((area.width as usize + CONNECTOR.chars().count()) / step).max(1);
    let offset = scroll_offset(app.stage_cursor, depth, visible);
    let end = (offset + visible).min(depth);

    app.geom.tree = area;
    app.geom.tree_offset = offset;
    app.geom.node_step = if area.height >= TREE_ROWS { step as u16 } else { 0 };

    if area.height < TREE_ROWS || area.width < node_w as u16 {
        let hint = Paragraph::new(Line::from(Span::styled(
            "  (terminal too small)",
            theme::dim(),
        )));
        f.render_widget(hint, area);
        return;
    }

    let rows = build_rows(word, app.stage_cursor, offset, end, node_w);

    let pad_top = (area.height.saturating_sub(TREE_ROWS) / 2) as usize;
    let mut lines: Vec<Line> = Vec::with_capacity(pad_top + rows.len());
    lines.extend(std::iter::repeat_with(Line::default).take(pad_top));
    lines.extend(rows);

    f.render_widget(Paragraph::new(lines), area);
}

/// Build the three rows (form / language / period) for the visible slice.
fn build_rows(
    word: &Word,
    cursor: usize,
    offset: usize,
    end: usize,
    node_w: usize,
) -> Vec<Line<'static>> {
    let depth = word.depth();
    let mut row: [Vec<Span>; 3] = Default::default();

    for i in offset..end {
        let layer = &word.chain[i];
        let ratio = if depth <= 1 {
            0.0
        } else {
            i as f32 / (depth - 1) as f32
        };
        let active = i == cursor;
        let form_style = if active {
            theme::selected()
        } else {
            Style::default().fg(theme::stage_color(ratio))
        };

        row[0].push(Span::styled(center(&layer.form, node_w), form_style));
        row[1].push(Span::styled(
            center(layer.language.label(), node_w),
            theme::dim(),
        ));
        row[2].push(Span::styled(
            center(&layer.period.label, node_w),
            theme::dim(),
        ));

        if i < depth - 1 && i < end - 1 {
            let arrow_style = Style::default().fg(theme::DIM);
            let blank = " ".repeat(CONNECTOR.chars().count());
            row[0].push(Span::styled(CONNECTOR, arrow_style));
            row[1].push(Span::styled(blank.clone(), arrow_style));
            row[2].push(Span::styled(blank, arrow_style));
        }
    }

    row.into_iter().map(Line::from).collect()
}

/// Center `s` within `width` columns, truncating with an ellipsis if needed.
fn center(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len > width {
        let take = width.saturating_sub(1);
        let truncated: String = s.chars().take(take).collect();
        return format!("{truncated}…");
    }
    let total = width - len;
    let left = total / 2;
    let right = total - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_offset_keeps_cursor_in_view() {
        assert_eq!(scroll_offset(0, 10, 3), 0);
        assert_eq!(scroll_offset(9, 10, 3), 7); // clamped to last page
        assert_eq!(scroll_offset(5, 10, 3), 4); // roughly centred
        assert_eq!(scroll_offset(5, 3, 10), 0); // everything fits
    }

    #[test]
    fn center_pads_and_truncates() {
        assert_eq!(center("ab", 6), "  ab  ");
        assert_eq!(center("toolongword", 5).chars().count(), 5);
        assert!(center("toolongword", 5).ends_with('…'));
    }
}
