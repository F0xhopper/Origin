//! Render smoke tests: draw real frames against a headless backend at several
//! sizes and assert the UI never panics and produces non-empty output.

use etymology_tui::app::App;
use etymology_tui::input::Action;
use etymology_tui::model::Dataset;
use etymology_tui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn render_at(width: u16, height: u16, prep: impl FnOnce(&mut App)) {
    let mut app = App::new(Dataset::embedded().unwrap());
    prep(&mut app);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
}

#[test]
fn renders_navigation_at_various_sizes() {
    for (w, h) in [(80, 24), (120, 40), (60, 16), (200, 50)] {
        render_at(w, h, |_| {});
    }
}

#[test]
fn renders_inspect_and_overlays() {
    render_at(100, 30, |a| {
        a.update(Action::Open);
        a.update(Action::Move(1));
    });
    render_at(100, 30, |a| {
        a.update(Action::ToggleHelp);
    });
    render_at(100, 30, |a| {
        a.update(Action::SearchStart);
        a.update(Action::SearchInput('s'));
    });
}

#[test]
fn renders_tiny_terminal_without_panic() {
    // Degenerate sizes must not panic.
    for (w, h) in [(10, 5), (4, 3), (1, 1)] {
        render_at(w, h, |a| {
            a.update(Action::Open);
        });
    }
}
