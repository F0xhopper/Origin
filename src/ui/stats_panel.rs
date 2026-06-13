//! Global stats panel: session-wide discovery progress.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

/// Draw the global stats panel.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(" Discovery ", theme::title()))
        .borders(Borders::ALL)
        .border_style(theme::border(app.focus_stats));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let s = &app.stats;
    let deepest = s
        .deepest()
        .map(|p| p.label.clone())
        .unwrap_or_else(|| "—".to_string());
    let total_words = app.dataset.len();

    let stat = |label: &str, value: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{label:<10}"), theme::dim()),
            Span::styled(
                value,
                Style::default()
                    .fg(theme::GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    };

    let lines = vec![
        stat("words", format!("{}/{}", s.words_explored(), total_words)),
        stat("layers", s.layers_visited().to_string()),
        stat("languages", s.languages_count().to_string()),
        stat("deepest", deepest),
        stat("session", s.elapsed_formatted()),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
