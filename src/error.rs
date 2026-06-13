//! Structured domain errors for the etymology TUI.

use thiserror::Error;

/// Errors that can arise while loading or validating the dataset.
#[derive(Debug, Error)]
pub enum AppError {
    /// Failed to read a dataset file from disk.
    #[error("failed to read dataset file `{path}`: {source}")]
    DataLoad {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse dataset JSON.
    #[error("failed to parse dataset JSON: {0}")]
    Parse(#[from] serde_json::Error),

    /// A word in the dataset had an empty etymology chain.
    #[error("word `{0}` has an empty etymology chain")]
    EmptyChain(String),

    /// The dataset contained no words at all.
    #[error("dataset is empty: no words to explore")]
    EmptyDataset,

    /// Generic terminal / IO failure.
    #[error("terminal io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience result alias for the crate.
pub type Result<T> = std::result::Result<T, AppError>;
