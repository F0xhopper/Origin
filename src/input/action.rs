//! Intent layer: keys and mouse events are translated into [`Action`]s, which
//! are the only thing [`crate::app::App::update`] knows about. This decouples
//! bindings from logic and makes the state machine testable without a terminal.

/// A high-level user intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Move the list/timeline cursor by a signed amount (honours vim counts).
    /// Positive is "down/deeper", negative is "up/shallower".
    Move(i32),
    /// Jump to the first item / oldest origin.
    JumpStart,
    /// Jump to the last item / modern form.
    JumpEnd,
    /// Open the currently selected word (Navigation -> Inspect).
    Open,
    /// Close the open word (Inspect -> Navigation).
    Back,
    /// Open a random word.
    Random,
    /// Toggle automatic backward traversal.
    ToggleAutoplay,
    /// Toggle stats focus.
    ToggleStats,
    /// Toggle the help overlay.
    ToggleHelp,
    /// Begin search.
    SearchStart,
    /// Append a character to the active search query.
    SearchInput(char),
    /// Delete the last character of the search query.
    SearchBackspace,
    /// Confirm the search, keeping the filter.
    SearchConfirm,
    /// Cancel the search, clearing the filter.
    SearchCancel,
    /// Cycle to the next search match.
    SearchNext,
    /// Cycle to the previous search match.
    SearchPrev,
    /// Select a word list row by absolute index (mouse).
    SelectWord(usize),
    /// Jump to a timeline stage by absolute index (mouse).
    SelectStage(usize),
    /// Quit the application.
    Quit,
}
