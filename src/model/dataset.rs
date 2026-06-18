//! Dataset loading, validation, indexing and search.

use std::path::Path;

use crate::error::{AppError, Result};
use crate::model::word::Word;

/// The embedded fallback dataset, baked into the binary so the app always works.
const EMBEDDED: &str = include_str!("../../assets/words.json");

/// A validated collection of words to explore.
#[derive(Debug, Clone)]
pub struct Dataset {
    words: Vec<Word>,
}

impl Dataset {
    /// Build a dataset from a JSON string, validating invariants.
    pub fn from_json(json: &str) -> Result<Self> {
        let words: Vec<Word> = serde_json::from_str(json)?;
        Self::from_words(words)
    }

    /// Build from already-parsed words, validating invariants and sorting by id.
    pub fn from_words(mut words: Vec<Word>) -> Result<Self> {
        if words.is_empty() {
            return Err(AppError::EmptyDataset);
        }
        for w in &words {
            if w.chain.is_empty() {
                return Err(AppError::EmptyChain(w.id.clone()));
            }
        }
        words.sort_by(|a, b| a.headword.to_lowercase().cmp(&b.headword.to_lowercase()));
        Ok(Self { words })
    }

    /// The embedded fallback dataset.
    pub fn embedded() -> Result<Self> {
        Self::from_json(EMBEDDED)
    }

    /// Load following the precedence: explicit path -> ./words.json -> embedded.
    pub fn load(explicit: Option<&Path>) -> Result<Self> {
        if let Some(path) = explicit {
            let json = std::fs::read_to_string(path).map_err(|source| AppError::DataLoad {
                path: path.display().to_string(),
                source,
            })?;
            return Self::from_json(&json);
        }
        let cwd_path = Path::new("words.json");
        if cwd_path.exists() {
            let json = std::fs::read_to_string(cwd_path).map_err(|source| AppError::DataLoad {
                path: cwd_path.display().to_string(),
                source,
            })?;
            return Self::from_json(&json);
        }
        Self::embedded()
    }

    /// All words, sorted by headword.
    pub fn words(&self) -> &[Word] {
        &self.words
    }

    /// Number of words.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Whether the dataset has no words (never true after validation, but kept
    /// for API completeness / clippy).
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Borrow a word by index.
    pub fn get(&self, idx: usize) -> Option<&Word> {
        self.words.get(idx)
    }

    /// Find a word index by its id (case-insensitive).
    pub fn index_of_id(&self, id: &str) -> Option<usize> {
        let id = id.to_lowercase();
        self.words.iter().position(|w| w.id.to_lowercase() == id)
    }

    /// Find a word index by its headword (case-insensitive).
    pub fn index_of_headword(&self, name: &str) -> Option<usize> {
        let name = name.trim().to_lowercase();
        self.words
            .iter()
            .position(|w| w.headword.to_lowercase() == name)
    }

    /// Resolve a user-typed query to a single word.
    ///
    /// Tries, in order: an exact id match, an exact headword match, then a
    /// unique substring match. Returns `None` when nothing matches or the
    /// match is ambiguous (use [`Dataset::search`] for suggestions).
    pub fn resolve(&self, query: &str) -> Option<usize> {
        let q = query.trim();
        if q.is_empty() {
            return None;
        }
        self.index_of_id(q)
            .or_else(|| self.index_of_headword(q))
            .or_else(|| {
                let hits = self.search(q);
                (hits.len() == 1).then(|| hits[0])
            })
    }

    /// Indices of words whose headword or id contains `query` (case-insensitive).
    /// An empty query matches everything.
    pub fn search(&self, query: &str) -> Vec<usize> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return (0..self.words.len()).collect();
        }
        self.words
            .iter()
            .enumerate()
            .filter(|(_, w)| {
                w.headword.to_lowercase().contains(&q) || w.id.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_dataset_loads_and_validates() {
        let ds = Dataset::embedded().expect("embedded dataset must parse");
        assert!(ds.len() >= 40, "expected >= 40 words, got {}", ds.len());
        for w in ds.words() {
            assert!(!w.chain.is_empty(), "word {} had empty chain", w.id);
            assert!(!w.headword.is_empty());
            assert!(!w.modern_meaning.is_empty());
        }
    }

    #[test]
    fn empty_dataset_is_rejected() {
        assert!(matches!(
            Dataset::from_words(vec![]),
            Err(AppError::EmptyDataset)
        ));
    }

    #[test]
    fn search_is_case_insensitive_and_substring() {
        let ds = Dataset::embedded().unwrap();
        let all = ds.search("");
        assert_eq!(all.len(), ds.len());
        let hits = ds.search("SAL");
        assert!(
            hits.iter().any(|&i| ds.get(i).unwrap().id == "salary"),
            "expected salary in results for 'SAL'"
        );
    }

    #[test]
    fn resolve_matches_id_headword_and_unique_substring() {
        let ds = Dataset::embedded().unwrap();
        let salary = ds.index_of_id("salary").unwrap();
        assert_eq!(ds.resolve("salary"), Some(salary));
        assert_eq!(ds.resolve("Salary"), Some(salary)); // headword, case-insensitive
        assert_eq!(ds.resolve("salar"), Some(salary)); // unique substring
        assert_eq!(ds.resolve("zzzznotaword"), None); // no match
        assert_eq!(ds.resolve(""), None); // empty
    }

    #[test]
    fn words_are_sorted_by_headword() {
        let ds = Dataset::embedded().unwrap();
        let names: Vec<String> = ds
            .words()
            .iter()
            .map(|w| w.headword.to_lowercase())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
