//! Application state and the pure-ish update logic that drives it.

use std::time::{Duration, Instant};

use crossterm::event::{MouseEvent, MouseEventKind};
use rand::Rng;
use ratatui::layout::Rect;

use crate::input::keymap::Pending;
use crate::input::Action;
use crate::model::Dataset;
use crate::stats::GlobalStats;

/// Interval between automatic backward steps during autoplay.
pub const AUTOPLAY_INTERVAL: Duration = Duration::from_millis(750);

/// Top-level interaction mode. Always shown in the status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Browsing the word list.
    Navigation,
    /// Exploring a single word's timeline.
    Inspect,
}

impl Mode {
    /// Short label for the status line.
    pub fn label(self) -> &'static str {
        match self {
            Mode::Navigation => "NAVIGATION",
            Mode::Inspect => "INSPECT",
        }
    }
}

/// A modal overlay floating above the main view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Search,
    Help,
}

/// Whether the event loop should keep running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}

/// Autoplay state: when the last automatic step occurred.
#[derive(Debug, Clone, Copy)]
struct AutoPlay {
    last_step: Instant,
}

/// Geometry captured during rendering so mouse clicks can be hit-tested.
#[derive(Debug, Default, Clone, Copy)]
pub struct Geometry {
    /// Inner content area of the word list (rows only).
    pub word_list: Rect,
    /// First visible word-list position (scroll offset into `filtered`).
    pub word_list_offset: usize,
    /// Inner content area of the timeline.
    pub timeline: Rect,
    /// First visible stage index (scroll offset into the chain).
    pub timeline_offset: usize,
    /// Vertical pixels per timeline stage (stages are spaced for connectors).
    pub timeline_step: u16,
}

/// The whole application state.
pub struct App {
    pub dataset: Dataset,
    /// Dataset indices currently shown, after the active search filter.
    pub filtered: Vec<usize>,
    /// Selection within `filtered`.
    pub list_cursor: usize,
    /// Open word as a dataset index; `Some` implies Inspect mode.
    pub open_word: Option<usize>,
    /// Cursor within the open word's chain (0 = modern, larger = older).
    pub stage_cursor: usize,
    pub mode: Mode,
    pub overlay: Overlay,
    pub search_query: String,
    pub focus_stats: bool,
    pub pending: Pending,
    pub stats: GlobalStats,
    pub geom: Geometry,
    autoplay: Option<AutoPlay>,
    flow: Flow,
}

impl App {
    /// Build a new app over a dataset.
    pub fn new(dataset: Dataset) -> Self {
        let filtered = (0..dataset.len()).collect();
        Self {
            dataset,
            filtered,
            list_cursor: 0,
            open_word: None,
            stage_cursor: 0,
            mode: Mode::Navigation,
            overlay: Overlay::None,
            search_query: String::new(),
            focus_stats: false,
            pending: Pending::default(),
            stats: GlobalStats::new(),
            geom: Geometry::default(),
            autoplay: None,
            flow: Flow::Continue,
        }
    }

    /// Whether the loop should keep running.
    pub fn running(&self) -> bool {
        self.flow == Flow::Continue
    }

    /// Whether autoplay is currently active.
    pub fn autoplay_active(&self) -> bool {
        self.autoplay.is_some()
    }

    /// The dataset index of the currently selected list row, if any.
    pub fn selected_index(&self) -> Option<usize> {
        self.filtered.get(self.list_cursor).copied()
    }

    /// The currently open word, if in Inspect mode.
    pub fn open(&self) -> Option<&crate::model::Word> {
        self.open_word.and_then(|i| self.dataset.get(i))
    }

    /// Apply an action to the state.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Move(delta) => self.move_cursor(delta),
            Action::JumpStart => self.jump_start(),
            Action::JumpEnd => self.jump_end(),
            Action::Open => self.open_selected(),
            Action::Back => self.close_word(),
            Action::Random => self.open_random(),
            Action::ToggleAutoplay => self.toggle_autoplay(),
            Action::ToggleStats => self.focus_stats = !self.focus_stats,
            Action::ToggleHelp => self.toggle_help(),
            Action::SearchStart => self.start_search(),
            Action::SearchInput(c) => self.search_input(c),
            Action::SearchBackspace => self.search_backspace(),
            Action::SearchConfirm => self.overlay = Overlay::None,
            Action::SearchCancel => self.cancel_search(),
            Action::SearchNext => self.cycle_match(1),
            Action::SearchPrev => self.cycle_match(-1),
            Action::SelectWord(pos) => self.select_word(pos),
            Action::SelectStage(stage) => self.select_stage(stage),
            Action::Quit => self.flow = Flow::Quit,
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        match self.mode {
            Mode::Navigation => {
                if self.filtered.is_empty() {
                    return;
                }
                let max = self.filtered.len() as i32 - 1;
                let next = (self.list_cursor as i32 + delta).clamp(0, max);
                self.list_cursor = next as usize;
            }
            Mode::Inspect => {
                if let Some(word) = self.open() {
                    let max = word.depth() as i32 - 1;
                    let next = (self.stage_cursor as i32 + delta).clamp(0, max);
                    self.stage_cursor = next as usize;
                    self.record_current();
                }
            }
        }
    }

    fn jump_start(&mut self) {
        match self.mode {
            Mode::Navigation => self.list_cursor = 0,
            Mode::Inspect => {
                if let Some(word) = self.open() {
                    self.stage_cursor = word.depth() - 1; // oldest origin
                    self.record_current();
                }
            }
        }
    }

    fn jump_end(&mut self) {
        match self.mode {
            Mode::Navigation => {
                if !self.filtered.is_empty() {
                    self.list_cursor = self.filtered.len() - 1;
                }
            }
            Mode::Inspect => {
                self.stage_cursor = 0; // modern
                self.record_current();
            }
        }
    }

    fn open_selected(&mut self) {
        if let Some(idx) = self.selected_index() {
            self.open_index(idx);
        }
    }

    fn open_random(&mut self) {
        if self.dataset.is_empty() {
            return;
        }
        let idx = rand::thread_rng().gen_range(0..self.dataset.len());
        // Reflect the choice in the list selection when it is visible.
        if let Some(pos) = self.filtered.iter().position(|&i| i == idx) {
            self.list_cursor = pos;
        }
        self.open_index(idx);
    }

    fn open_index(&mut self, idx: usize) {
        self.open_word = Some(idx);
        self.mode = Mode::Inspect;
        self.stage_cursor = 0;
        self.autoplay = None;
        if let Some(word) = self.dataset.get(idx) {
            self.stats.record_word(&word.id);
        }
        self.record_current();
    }

    fn close_word(&mut self) {
        self.open_word = None;
        self.mode = Mode::Navigation;
        self.autoplay = None;
    }

    fn toggle_autoplay(&mut self) {
        if self.mode != Mode::Inspect {
            return;
        }
        self.autoplay = match self.autoplay {
            Some(_) => None,
            None => Some(AutoPlay {
                last_step: Instant::now(),
            }),
        };
    }

    fn toggle_help(&mut self) {
        self.overlay = if self.overlay == Overlay::Help {
            Overlay::None
        } else {
            Overlay::Help
        };
    }

    fn start_search(&mut self) {
        self.overlay = Overlay::Search;
        self.search_query.clear();
        self.refilter();
    }

    fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.refilter();
    }

    fn search_backspace(&mut self) {
        self.search_query.pop();
        self.refilter();
    }

    fn cancel_search(&mut self) {
        self.overlay = Overlay::None;
        self.search_query.clear();
        self.refilter();
    }

    /// Recompute the filtered list and keep the cursor in range.
    fn refilter(&mut self) {
        self.filtered = self.dataset.search(&self.search_query);
        if self.filtered.is_empty() {
            self.list_cursor = 0;
        } else if self.list_cursor >= self.filtered.len() {
            self.list_cursor = self.filtered.len() - 1;
        }
    }

    fn cycle_match(&mut self, dir: i32) {
        if self.filtered.is_empty() {
            return;
        }
        let len = self.filtered.len() as i32;
        let next = (self.list_cursor as i32 + dir).rem_euclid(len);
        self.list_cursor = next as usize;
    }

    fn select_word(&mut self, pos: usize) {
        if pos < self.filtered.len() {
            self.list_cursor = pos;
            if self.mode == Mode::Inspect {
                // Clicking the list while inspecting opens the clicked word.
                if let Some(idx) = self.selected_index() {
                    self.open_index(idx);
                }
            }
        }
    }

    fn select_stage(&mut self, stage: usize) {
        if let Some(word) = self.open() {
            if stage < word.depth() {
                self.stage_cursor = stage;
                self.record_current();
            }
        }
    }

    /// Record stats for the currently focused layer.
    fn record_current(&mut self) {
        if let Some(word) = self.open_word.and_then(|i| self.dataset.get(i)) {
            if let Some(layer) = word.chain.get(self.stage_cursor) {
                let (id, lang, period) = (
                    word.id.clone(),
                    layer.language.clone(),
                    layer.period.clone(),
                );
                self.stats
                    .record_layer(&id, self.stage_cursor, &lang, &period);
            }
        }
    }

    /// Advance autoplay if it is time. Called once per loop tick.
    pub fn tick(&mut self) {
        let Some(state) = self.autoplay else { return };
        if state.last_step.elapsed() < AUTOPLAY_INTERVAL {
            return;
        }
        let Some(word) = self.open() else {
            self.autoplay = None;
            return;
        };
        if self.stage_cursor + 1 >= word.depth() {
            // Reached the oldest root: stop.
            self.autoplay = None;
            return;
        }
        self.stage_cursor += 1;
        self.record_current();
        self.autoplay = Some(AutoPlay {
            last_step: Instant::now(),
        });
    }

    /// Handle a mouse event using the geometry captured during the last draw.
    pub fn handle_mouse(&mut self, ev: MouseEvent) {
        match ev.kind {
            MouseEventKind::ScrollDown => self.update(Action::Move(1)),
            MouseEventKind::ScrollUp => self.update(Action::Move(-1)),
            MouseEventKind::Down(_) => {
                if hit(self.geom.word_list, ev.column, ev.row) {
                    let row = (ev.row - self.geom.word_list.y) as usize;
                    let pos = self.geom.word_list_offset + row;
                    self.update(Action::SelectWord(pos));
                } else if self.mode == Mode::Inspect
                    && self.geom.timeline_step > 0
                    && hit(self.geom.timeline, ev.column, ev.row)
                {
                    let row = (ev.row - self.geom.timeline.y) / self.geom.timeline_step;
                    let stage = self.geom.timeline_offset + row as usize;
                    self.update(Action::SelectStage(stage));
                }
            }
            _ => {}
        }
    }
}

/// Whether a point falls inside a rect.
fn hit(rect: Rect, col: u16, row: u16) -> bool {
    rect.width > 0
        && rect.height > 0
        && col >= rect.x
        && col < rect.x + rect.width
        && row >= rect.y
        && row < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(Dataset::embedded().unwrap())
    }

    #[test]
    fn navigation_move_clamps() {
        let mut a = app();
        a.update(Action::Move(-5));
        assert_eq!(a.list_cursor, 0);
        a.update(Action::JumpEnd);
        assert_eq!(a.list_cursor, a.filtered.len() - 1);
        a.update(Action::Move(10));
        assert_eq!(a.list_cursor, a.filtered.len() - 1);
    }

    #[test]
    fn open_and_traverse_timeline() {
        let mut a = app();
        a.update(Action::Open);
        assert_eq!(a.mode, Mode::Inspect);
        assert_eq!(a.stage_cursor, 0);
        let depth = a.open().unwrap().depth();
        // gg jumps to oldest origin.
        a.update(Action::JumpStart);
        assert_eq!(a.stage_cursor, depth - 1);
        // G returns to modern.
        a.update(Action::JumpEnd);
        assert_eq!(a.stage_cursor, 0);
        // h (deeper) clamps at the root.
        a.update(Action::Move(100));
        assert_eq!(a.stage_cursor, depth - 1);
        // Esc closes.
        a.update(Action::Back);
        assert_eq!(a.mode, Mode::Navigation);
        assert!(a.open_word.is_none());
    }

    #[test]
    fn search_filters_and_cycles() {
        let mut a = app();
        a.update(Action::SearchStart);
        for c in "sal".chars() {
            a.update(Action::SearchInput(c));
        }
        assert!(!a.filtered.is_empty());
        assert!(a.filtered.iter().all(|&i| a
            .dataset
            .get(i)
            .unwrap()
            .headword
            .to_lowercase()
            .contains("sal")
            || a.dataset.get(i).unwrap().id.contains("sal")));
        a.update(Action::SearchConfirm);
        // Cycling wraps around.
        let start = a.list_cursor;
        for _ in 0..a.filtered.len() {
            a.update(Action::SearchNext);
        }
        assert_eq!(a.list_cursor, start);
    }

    #[test]
    fn opening_a_word_records_stats() {
        let mut a = app();
        a.update(Action::Open);
        assert_eq!(a.stats.words_explored(), 1);
        assert!(a.stats.layers_visited() >= 1);
        a.update(Action::JumpStart);
        assert!(a.stats.layers_visited() >= 2);
    }
}
