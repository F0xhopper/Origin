//! Command-line arguments.

use std::path::PathBuf;

use clap::Parser;

/// A terminal-based linguistic time machine: peel modern words backward
/// through their etymological history.
#[derive(Debug, Parser)]
#[command(name = "etymology-tui", version, about)]
pub struct Cli {
    /// Load an external words.json dataset.
    #[arg(long, value_name = "PATH")]
    pub data: Option<PathBuf>,

    /// Open directly into a word's timeline on startup (by id).
    #[arg(long, value_name = "ID")]
    pub word: Option<String>,

    /// Start on a random word.
    #[arg(long)]
    pub random: bool,

    /// Disable mouse capture.
    #[arg(long)]
    pub no_mouse: bool,

    /// Enable logging to a file.
    #[arg(long, value_name = "PATH")]
    pub log: Option<PathBuf>,
}
