//! Detail panel: the active node of the chain, in depth.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

/// Draw details for the currently focused node.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Stage ", theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(false));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(layer) = app.word.chain.get(app.stage_cursor) {
        let mut head = field("Form", &layer.form);
        head.push(Span::raw("    "));
        head.extend(field("Language", layer.language.label()));
        head.push(Span::raw("    "));
        head.extend(field("Period", &layer.period.label));
        lines.push(Line::from(head));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("Meaning  ", theme::dim()),
            Span::styled(layer.meaning.clone(), theme::text()),
        ]));
        if let Some(note) = &layer.note {
            lines.push(Line::from(vec![
                Span::styled("Note     ", theme::dim()),
                Span::styled(note.clone(), theme::text()),
            ]));
        }
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            format!("stage {}/{}", app.stage_cursor + 1, app.depth()),
            theme::dim(),
        )));
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn field(label: &str, value: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{label}: "), theme::dim()),
        Span::styled(value.to_string(), theme::text()),
    ]
}
