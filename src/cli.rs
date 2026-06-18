//! Command-line arguments.

use std::path::PathBuf;

use clap::Parser;

/// Explore a word's origin: `origin <word>` opens a horizontal, vim-navigable
/// etymology tree, tracing the word backward to its ancient root.
#[derive(Debug, Parser)]
#[command(name = "origin", version, about)]
pub struct Cli {
    /// The word to trace (e.g. `origin salary`).
    #[arg(value_name = "WORD")]
    pub word: Option<String>,

    /// Load an external words.json dataset.
    #[arg(long, value_name = "PATH")]
    pub data: Option<PathBuf>,

    /// Disable mouse capture.
    #[arg(long)]
    pub no_mouse: bool,

    /// Enable logging to a file.
    #[arg(long, value_name = "PATH")]
    pub log: Option<PathBuf>,
}
