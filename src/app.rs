//! Application state and the pure-ish update logic that drives it.
//!
//! The app explores a single word: its etymology chain laid out left (modern)
//! to right (oldest root), with a cursor the user walks back through time.

use std::time::{Duration, Instant};

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::input::keymap::Pending;
use crate::input::Action;
use crate::model::Word;

/// Interval between automatic steps toward the root during autoplay.
pub const AUTOPLAY_INTERVAL: Duration = Duration::from_millis(750);

/// A modal overlay floating above the main view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
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
    /// Inner content area of the tree.
    pub tree: Rect,
    /// First visible node index (horizontal scroll offset into the chain).
    pub tree_offset: usize,
    /// Horizontal columns per node (node box plus its trailing connector).
    pub node_step: u16,
}

/// The whole application state: one word and a cursor over its chain.
pub struct App {
    /// The word being explored, with its chain ordered modern -> oldest.
    pub word: Word,
    /// Cursor within the chain (0 = modern/left, depth-1 = root/right).
    pub stage_cursor: usize,
    pub overlay: Overlay,
    pub pending: Pending,
    pub geom: Geometry,
    autoplay: Option<AutoPlay>,
    flow: Flow,
}

impl App {
    /// Build an app focused on a single word, starting at its modern form.
    pub fn new(word: Word) -> Self {
        Self {
            word,
            stage_cursor: 0,
            overlay: Overlay::None,
            pending: Pending::default(),
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

    /// Number of layers in the chain.
    pub fn depth(&self) -> usize {
        self.word.depth()
    }

    /// Apply an action to the state.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Move(delta) => self.move_cursor(delta),
            Action::JumpModern => self.stage_cursor = 0,
            Action::JumpRoot => self.stage_cursor = self.depth().saturating_sub(1),
            Action::ToggleAutoplay => self.toggle_autoplay(),
            Action::ToggleHelp => self.toggle_help(),
            Action::SelectStage(stage) => self.select_stage(stage),
            Action::Quit => self.flow = Flow::Quit,
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        let max = self.depth() as i32 - 1;
        let next = (self.stage_cursor as i32 + delta).clamp(0, max);
        self.stage_cursor = next as usize;
    }

    fn toggle_autoplay(&mut self) {
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

    fn select_stage(&mut self, stage: usize) {
        if stage < self.depth() {
            self.stage_cursor = stage;
        }
    }

    /// Advance autoplay if it is time. Called once per loop tick.
    pub fn tick(&mut self) {
        let Some(state) = self.autoplay else { return };
        if state.last_step.elapsed() < AUTOPLAY_INTERVAL {
            return;
        }
        if self.stage_cursor + 1 >= self.depth() {
            // Reached the oldest root: stop.
            self.autoplay = None;
            return;
        }
        self.stage_cursor += 1;
        self.autoplay = Some(AutoPlay {
            last_step: Instant::now(),
        });
    }

    /// Handle a mouse event using the geometry captured during the last draw.
    pub fn handle_mouse(&mut self, ev: MouseEvent) {
        match ev.kind {
            // Scrolling walks the cursor; down/right goes older.
            MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => {
                self.update(Action::Move(1))
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => self.update(Action::Move(-1)),
            MouseEventKind::Down(_) => {
                if self.geom.node_step > 0 && hit(self.geom.tree, ev.column, ev.row) {
                    let col = (ev.column - self.geom.tree.x) / self.geom.node_step;
                    let stage = self.geom.tree_offset + col as usize;
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
    use crate::model::Dataset;

    fn app() -> App {
        let ds = Dataset::embedded().unwrap();
        let idx = ds.resolve("salary").unwrap();
        App::new(ds.get(idx).unwrap().clone())
    }

    #[test]
    fn starts_at_modern_form() {
        let a = app();
        assert_eq!(a.stage_cursor, 0);
        assert!(a.depth() >= 2);
    }

    #[test]
    fn move_clamps_at_both_ends() {
        let mut a = app();
        a.update(Action::Move(-5));
        assert_eq!(a.stage_cursor, 0);
        a.update(Action::Move(100));
        assert_eq!(a.stage_cursor, a.depth() - 1);
    }

    #[test]
    fn jump_modern_and_root() {
        let mut a = app();
        a.update(Action::JumpRoot);
        assert_eq!(a.stage_cursor, a.depth() - 1);
        a.update(Action::JumpModern);
        assert_eq!(a.stage_cursor, 0);
    }

    #[test]
    fn select_stage_ignores_out_of_range() {
        let mut a = app();
        a.update(Action::SelectStage(1));
        assert_eq!(a.stage_cursor, 1);
        a.update(Action::SelectStage(9999));
        assert_eq!(a.stage_cursor, 1);
    }

    #[test]
    fn autoplay_toggles_and_stops_at_the_root() {
        let mut a = app();
        a.update(Action::ToggleAutoplay);
        assert!(a.autoplay_active());
        a.update(Action::ToggleAutoplay);
        assert!(!a.autoplay_active());

        // At the root with autoplay armed, the next due tick halts it.
        a.update(Action::JumpRoot);
        a.update(Action::ToggleAutoplay);
        std::thread::sleep(AUTOPLAY_INTERVAL + Duration::from_millis(20));
        a.tick();
        assert!(!a.autoplay_active(), "autoplay stops at the root");
    }
}
