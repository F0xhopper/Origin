//! Vim-style key translation. A pure function over the current overlay and a
//! small amount of pending chord/count state.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Overlay;
use crate::input::action::Action;

/// Transient multi-key state: numeric count prefix and the `g` of a `gg` chord.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pending {
    /// Accumulated numeric prefix, e.g. `3` for `3l`.
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
pub fn map_key(overlay: Overlay, key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    match overlay {
        Overlay::Help => map_help(key, pending),
        Overlay::None => map_tree(key, pending),
    }
}

fn map_help(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    pending.clear();
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => Some(Action::ToggleHelp),
        _ => None,
    }
}

fn map_tree(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    // The `gg` chord -> modern form.
    if let Some(action) = map_g_chord(key, pending) {
        return Some(action);
    }

    // Numeric count prefix (but a lone 0 jumps to the modern form).
    if let KeyCode::Char(c @ '0'..='9') = key.code {
        if !(c == '0' && pending.count.is_none()) {
            let digit = c as u32 - '0' as u32;
            pending.count = Some(pending.count.unwrap_or(0).saturating_mul(10) + digit);
            return None;
        }
    }

    match key.code {
        // Older / right.
        KeyCode::Char('l') | KeyCode::Right => Some(Action::Move(pending.take_count())),
        // Newer / left.
        KeyCode::Char('h') | KeyCode::Left => Some(Action::Move(-pending.take_count())),
        // Line-start / line-end vim motions map onto the chain ends.
        KeyCode::Char('0') | KeyCode::Home => {
            pending.clear();
            Some(Action::JumpModern)
        }
        KeyCode::Char('$') | KeyCode::Char('G') | KeyCode::End => {
            pending.clear();
            Some(Action::JumpRoot)
        }
        KeyCode::Char(' ') => {
            pending.clear();
            Some(Action::ToggleAutoplay)
        }
        KeyCode::Char('?') => {
            pending.clear();
            Some(Action::ToggleHelp)
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            pending.clear();
            Some(Action::Quit)
        }
        _ => None,
    }
}

/// Handle the `gg` chord. Returns `Some(JumpModern)` on the second `g`.
fn map_g_chord(key: KeyEvent, pending: &mut Pending) -> Option<Action> {
    if key.code == KeyCode::Char('g') && !key.modifiers.contains(KeyModifiers::SHIFT) {
        if pending.g {
            pending.clear();
            return Some(Action::JumpModern);
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
    fn h_is_newer_l_is_older() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Overlay::None, k('l'), &mut p),
            Some(Action::Move(1))
        );
        assert_eq!(
            map_key(Overlay::None, k('h'), &mut p),
            Some(Action::Move(-1))
        );
    }

    #[test]
    fn count_prefix_multiplies_movement() {
        let mut p = Pending::default();
        assert_eq!(map_key(Overlay::None, k('3'), &mut p), None);
        assert_eq!(
            map_key(Overlay::None, k('l'), &mut p),
            Some(Action::Move(3))
        );
        // Count is consumed.
        assert_eq!(
            map_key(Overlay::None, k('l'), &mut p),
            Some(Action::Move(1))
        );
    }

    #[test]
    fn gg_chord_jumps_to_modern() {
        let mut p = Pending::default();
        assert_eq!(map_key(Overlay::None, k('g'), &mut p), None);
        assert!(p.g);
        assert_eq!(
            map_key(Overlay::None, k('g'), &mut p),
            Some(Action::JumpModern)
        );
        assert!(!p.g);
    }

    #[test]
    fn capital_g_and_dollar_jump_to_root() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Overlay::None, k('G'), &mut p),
            Some(Action::JumpRoot)
        );
        assert_eq!(
            map_key(Overlay::None, k('$'), &mut p),
            Some(Action::JumpRoot)
        );
    }

    #[test]
    fn lone_zero_jumps_to_modern_but_extends_a_count() {
        let mut p = Pending::default();
        assert_eq!(
            map_key(Overlay::None, k('0'), &mut p),
            Some(Action::JumpModern)
        );
        // 1 then 0 makes a count of 10, not a jump.
        map_key(Overlay::None, k('1'), &mut p);
        assert_eq!(map_key(Overlay::None, k('0'), &mut p), None);
        assert_eq!(p.count, Some(10));
    }

    #[test]
    fn q_and_esc_quit() {
        let mut p = Pending::default();
        assert_eq!(map_key(Overlay::None, k('q'), &mut p), Some(Action::Quit));
        assert_eq!(
            map_key(
                Overlay::None,
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                &mut p
            ),
            Some(Action::Quit)
        );
    }
}
