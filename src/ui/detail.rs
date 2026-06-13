//! Right-column detail panel: the focused historical stage in depth.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::ui::theme;

/// Draw details for the active stage (Inspect) or selected word (Navigation).
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Stage ", theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(false));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if app.mode == Mode::Inspect {
        if let Some(word) = app.open() {
            if let Some(layer) = word.chain.get(app.stage_cursor) {
                lines.push(field("Form", &layer.form));
                lines.push(field("Language", layer.language.label()));
                lines.push(field("Period", &layer.period.label));
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled("Meaning", theme::dim())));
                lines.push(Line::from(Span::styled(
                    layer.meaning.clone(),
                    theme::text(),
                )));
                if let Some(note) = &layer.note {
                    lines.push(Line::raw(""));
                    lines.push(Line::from(Span::styled("Note", theme::dim())));
                    lines.push(Line::from(Span::styled(note.clone(), theme::text())));
                }
                let pos = format!("stage {}/{}", app.stage_cursor + 1, word.depth());
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled(pos, theme::dim())));
            }
        }
    } else if let Some(idx) = app.selected_index() {
        if let Some(word) = app.dataset.get(idx) {
            lines.push(field("Word", &word.headword));
            lines.push(field("Depth", &format!("{} layers", word.depth())));
            lines.push(field("Root", &word.deepest_period().label));
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled("Modern meaning", theme::dim())));
            lines.push(Line::from(Span::styled(
                word.modern_meaning.clone(),
                theme::text(),
            )));
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "press Enter to dive",
                theme::dim(),
            )));
        }
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn field(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::dim()),
        Span::styled(value.to_string(), theme::text()),
    ])
}
