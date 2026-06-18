//! `origin` — a terminal "linguistic time machine".
//!
//! Given a single word, the app lays out its etymology chain horizontally —
//! modern form on the left, ancient root on the right — and lets the user walk
//! back through time with Vim-style keys.
//!
//! The crate is split so that all logic (model, state machine, input mapping)
//! is pure and testable without a terminal; only [`tui`] and the binary touch
//! the real terminal.

pub mod app;
pub mod cli;
pub mod error;
pub mod input;
pub mod model;
pub mod tui;
pub mod ui;
