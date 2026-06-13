//! End-to-end exercises of the public API, driving the app via actions only.

use etymology_tui::app::{App, Mode};
use etymology_tui::input::Action;
use etymology_tui::model::Dataset;

fn app() -> App {
    App::new(Dataset::embedded().expect("embedded dataset"))
}

#[test]
fn reverse_dive_session_accumulates_discovery() {
    let mut a = app();

    // Open the first word and dive to its oldest root.
    a.update(Action::Open);
    assert_eq!(a.mode, Mode::Inspect);
    a.update(Action::JumpStart); // gg -> oldest origin
    let depth = a.open().unwrap().depth();
    assert_eq!(a.stage_cursor, depth - 1);

    // Back out and explore a second word.
    a.update(Action::Back);
    a.update(Action::Move(1));
    a.update(Action::Open);

    assert_eq!(a.stats.words_explored(), 2);
    assert!(a.stats.languages_count() >= 2);
    assert!(a.stats.deepest().is_some());
}

#[test]
fn discovery_mode_browses_without_opening() {
    let mut a = app();
    let last = a.filtered.len() - 1;
    a.update(Action::JumpEnd);
    assert_eq!(a.list_cursor, last);
    assert_eq!(a.mode, Mode::Navigation);
    // Browsing alone explores nothing.
    assert_eq!(a.stats.words_explored(), 0);
}

#[test]
fn deep_origin_then_compare_forward() {
    let mut a = app();
    a.update(Action::Open);
    a.update(Action::JumpStart); // deepest
    let deepest = a.open().unwrap().deepest_period().sort_key;
    assert_eq!(
        a.open().unwrap().chain[a.stage_cursor].period.sort_key,
        deepest
    );
    a.update(Action::JumpEnd); // back to modern
    assert_eq!(a.stage_cursor, 0);
}

#[test]
fn random_opens_some_word() {
    let mut a = app();
    a.update(Action::Random);
    assert_eq!(a.mode, Mode::Inspect);
    assert!(a.open().is_some());
}

#[test]
fn search_then_clear_restores_all() {
    let mut a = app();
    let total = a.filtered.len();
    a.update(Action::SearchStart);
    for c in "zzzznotaword".chars() {
        a.update(Action::SearchInput(c));
    }
    assert!(a.filtered.is_empty());
    a.update(Action::SearchCancel);
    assert_eq!(a.filtered.len(), total);
}
