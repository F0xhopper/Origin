//! Render smoke tests: draw real frames against a headless backend at several
//! sizes and assert the UI never panics and produces output.

use etymology_tui::app::App;
use etymology_tui::input::Action;
use etymology_tui::model::Dataset;
use etymology_tui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn app() -> App {
    let ds = Dataset::embedded().unwrap();
    let idx = ds.resolve("salary").unwrap();
    App::new(ds.get(idx).unwrap().clone())
}

fn render_at(width: u16, height: u16, prep: impl FnOnce(&mut App)) {
    let mut app = app();
    prep(&mut app);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
}

#[test]
fn renders_tree_at_various_sizes() {
    for (w, h) in [(80, 24), (120, 40), (60, 16), (200, 50)] {
        render_at(w, h, |a| {
            a.update(Action::Move(2));
        });
    }
}

#[test]
fn renders_help_overlay() {
    render_at(100, 30, |a| {
        a.update(Action::ToggleHelp);
    });
}

#[test]
fn renders_tiny_terminal_without_panic() {
    // Degenerate sizes must not panic.
    for (w, h) in [(10, 5), (4, 3), (1, 1)] {
        render_at(w, h, |a| {
            a.update(Action::JumpRoot);
        });
    }
}
