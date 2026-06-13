//! Vim-style modal key translation. Pure function over the current mode,
//! overlay and a small amount of pending chord/count state.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Mode, Overlay};
use crate::input::action::Action;

/// Transient multi-key state: numeric count prefix and the `g` of a `gg` chord.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pending {
    /// Accumulated numeric prefix, e.g. `3` for `3j`.
    pub count: Option<u32>,
    /// True after the first `g` of a `gg` chord.
    pub g: bool,
}

impl Pending {
    /// Take the pending count (default 1) and clear it.
    fn take_count(&mut self) -> i32 {
        let n = self.count.take().unwrap_or(1);
        n.clamp(1, 9999) as i32
    }

    /// Clear all pending chord/count state.
    pub fn clear(&mut self) {
        self.count = None;
        self.g = false;
    }
}

/// Translate a key event into an [`Action`], updating `pending` for chords and
/// counts. Returns `None` when the key only mutated pending state (e.g. a digit
/// or the first `g`) or is unbound in the current context.
pub fn map_key(
    mode: Mode,
    overlay: Overlay,
    key: KeyEvent,
    pending: &mut Pending,
) -> Option<Action> {
    match overlay {
        Overlay::Help => map_help(key, pending),
        Overlay::Search => map_search(key),
        Overlay::None => match mode {
            Mode::Navigation => map_navigation(key, pending),
            Mode::Inspect => map_inspect(key, pending),
        },
    }
}

fn map_help(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    pending.clear();
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => Some(Action::ToggleHelp),
        _ => None,
    }
}

fn map_search(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::SearchCancel),
        KeyCode::Enter => Some(Action::SearchConfirm),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Char(c) => Some(Action::SearchInput(c)),
        _ => None,
    }
}

/// Handle keys common to Navigation and Inspect (overlay = None).
fn map_common(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    // Numeric count prefix (but a lone 0 is not a count).
    if let KeyCode::Char(c @ '0'..='9') = key.code {
        if !(c == '0' && pending.count.is_none()) {
            let digit = c as u32 - '0' as u32;
            pending.count = Some(pending.count.unwrap_or(0).saturating_mul(10) + digit);
            return None;
        }
    }

    match key.code {
        KeyCode::Char('q') => {
            pending.clear();
            Some(Action::Quit)
        }
        KeyCode::Char('?') => {
            pending.clear();
            Some(Action::ToggleHelp)
        }
        KeyCode::Char('s') => {
            pending.clear();
            Some(Action::ToggleStats)
        }
        KeyCode::Char('/') => {
            pending.clear();
            Some(Action::SearchStart)
        }
        KeyCode::Char('r') => {
            pending.clear();
            Some(Action::Random)
        }
        KeyCode::Char('n') => {
            pending.clear();
            Some(Action::SearchNext)
        }
        KeyCode::Char('N') => {
            pending.clear();
            Some(Action::SearchPrev)
        }
        _ => None,
    }
}

fn map_navigation(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    // The `gg` chord.
    if let Some(action) = map_g_chord(key, pending) {
        return Some(action);
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Move(pending.take_count())),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Move(-pending.take_count())),
        KeyCode::Char('G') => {
            pending.clear();
            Some(Action::JumpEnd)
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            pending.clear();
            Some(Action::Open)
        }
        KeyCode::Esc => {
            pending.clear();
            None
        }
        _ => map_common(key, pending),
    }
}

fn map_inspect(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    // The `gg` chord -> oldest origin.
    if let Some(action) = map_g_chord(key, pending) {
        return Some(action);
    }
    match key.code {
        // Deeper into history (older).
        KeyCode::Char('h') | KeyCode::Char('j') | KeyCode::Left | KeyCode::Down => {
            Some(Action::Move(pending.take_count()))
        }
        // Back toward modern.
        KeyCode::Char('l') | KeyCode::Char('k') | KeyCode::Right | KeyCode::Up => {
            Some(Action::Move(-pending.take_count()))
        }
        KeyCode::Char('G') => {
            pending.clear();
            Some(Action::JumpEnd)
        }
        KeyCode::Char(' ') => {
            pending.clear();
            Some(Action::ToggleAutoplay)
        }
        KeyCode::Esc => {
            pending.clear();
            Some(Action::Back)
        }
        _ => map_common(key, pending),
    }
}

/// Handle the `gg` chord. Returns `Some(JumpStart)` on the second `g`.
fn map_g_chord(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    if key.code == KeyCode::Char('g') && !key.modifiers.contains(KeyModifiers::SHIFT) {
        if pending.g {
            pending.clear();
            return Some(Action::JumpStart);
        }
        pending.g = true;
        return None;
    }
    // Any other key cancels a half-finished `g` chord (but still processes).
    pending.g = false;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn count_prefix_multiplies_movement() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('3'), &mut p),
            None
        );
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('j'), &mut p),
            Some(Action::Move(3))
        );
        // Count is consumed.
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('j'), &mut p),
            Some(Action::Move(1))
        );
    }

    #[test]
    fn gg_chord_jumps_to_start() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('g'), &mut p),
            None
        );
        assert!(p.g);
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('g'), &mut p),
            Some(Action::JumpStart)
        );
        assert!(!p.g);
    }

    #[test]
    fn inspect_h_is_deeper_l_is_shallower() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Mode::Inspect, Overlay::None, k('h'), &mut p),
            Some(Action::Move(1))
        );
        assert_eq!(
            map_key(Mode::Inspect, Overlay::None, k('l'), &mut p),
            Some(Action::Move(-1))
        );
    }

    #[test]
    fn search_overlay_captures_typing() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Mode::Navigation, Overlay::Search, k('a'), &mut p),
            Some(Action::SearchInput('a'))
        );
        assert_eq!(
            map_key(
                Mode::Navigation,
                Overlay::Search,
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                &mut p
            ),
            Some(Action::SearchCancel)
        );
    }

    #[test]
    fn lone_zero_is_not_a_count() {
        let mut p = Pending::default();
        // 0 with no pending count is unbound, not a count.
        assert_eq!(
            map_key(Mode::Navigation, Overlay::None, k('0'), &mut p),
            None
        );
        assert_eq!(p.count, None);
        // But 1 then 0 makes 10.
        map_key(Mode::Navigation, Overlay::None, k('1'), &mut p);
        map_key(Mode::Navigation, Overlay::None, k('0'), &mut p);
        assert_eq!(p.count, Some(10));
    }
}
