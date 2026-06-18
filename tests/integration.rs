//! End-to-end exercises of the public API, driving the app via actions only.

use etymology_tui::app::App;
use etymology_tui::input::Action;
use etymology_tui::model::Dataset;

fn app_for(query: &str) -> App {
    let ds = Dataset::embedded().expect("embedded dataset");
    let idx = ds.resolve(query).expect("query resolves");
    App::new(ds.get(idx).unwrap().clone())
}

#[test]
fn walks_back_to_the_root_and_returns() {
    let mut a = app_for("salary");
    assert_eq!(a.stage_cursor, 0, "starts at the modern form");

    a.update(Action::JumpRoot);
    assert_eq!(a.stage_cursor, a.depth() - 1);
    let root = a.word.deepest_period().sort_key;
    assert_eq!(a.word.chain[a.stage_cursor].period.sort_key, root);

    a.update(Action::JumpModern);
    assert_eq!(a.stage_cursor, 0);
}

#[test]
fn stepping_clamps_at_both_ends() {
    let mut a = app_for("salary");
    a.update(Action::Move(-3));
    assert_eq!(a.stage_cursor, 0);
    a.update(Action::Move(1000));
    assert_eq!(a.stage_cursor, a.depth() - 1);
}

#[test]
fn resolution_picks_the_right_word() {
    let ds = Dataset::embedded().unwrap();
    let by_id = ds.resolve("salary").unwrap();
    let by_headword = ds.resolve("Salary").unwrap();
    assert_eq!(by_id, by_headword);
    assert!(ds.resolve("zzzznotaword").is_none());
}
