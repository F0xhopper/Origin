//! Reverse Etymology Timeline TUI — library crate.
//!
//! A terminal "linguistic time machine": users start at a modern word and peel
//! it backward through historical layers to its proto-language roots.
//!
//! The crate is split so that all logic (model, state machine, input mapping,
//! stats) is pure and testable without a terminal; only [`tui`] and the binary
//! touch the real terminal.

pub mod app;
pub mod cli;
pub mod error;
pub mod input;
pub mod model;
pub mod stats;
pub mod tui;
pub mod ui;
