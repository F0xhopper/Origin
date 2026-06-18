//! Intent layer: keys and mouse events are translated into [`Action`]s, which
//! are the only thing [`crate::app::App::update`] knows about. This decouples
//! bindings from logic and makes the state machine testable without a terminal.

/// A high-level user intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Move the cursor along the tree by a signed amount (honours vim counts).
    /// Positive moves right/older, negative moves left/newer.
    Move(i32),
    /// Jump to the modern form (leftmost node).
    JumpModern,
    /// Jump to the oldest root (rightmost node).
    JumpRoot,
    /// Toggle automatic stepping toward the root.
    ToggleAutoplay,
    /// Toggle the help overlay.
    ToggleHelp,
    /// Jump to a tree node by absolute index (mouse).
    SelectStage(usize),
    /// Quit the application.
    Quit,
}
