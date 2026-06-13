//! Domain model: words, etymology layers, and the dataset.

pub mod dataset;
pub mod word;

pub use dataset::Dataset;
pub use word::{Language, Period, Word};
