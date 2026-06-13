//! Session-wide exploration statistics.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::model::{Language, Period};

/// Tracks the user's progress through linguistic history during a session.
#[derive(Debug)]
pub struct GlobalStats {
    words_explored: HashSet<String>,
    layers_visited: HashSet<(String, usize)>,
    languages: HashSet<Language>,
    deepest: Option<Period>,
    start: Instant,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStats {
    /// Create a fresh stats tracker, starting the session timer.
    pub fn new() -> Self {
        Self {
            words_explored: HashSet::new(),
            layers_visited: HashSet::new(),
            languages: HashSet::new(),
            deepest: None,
            start: Instant::now(),
        }
    }

    /// Record that a word was opened for exploration.
    pub fn record_word(&mut self, word_id: &str) {
        self.words_explored.insert(word_id.to_string());
    }

    /// Record that a specific layer of a word was viewed.
    pub fn record_layer(
        &mut self,
        word_id: &str,
        stage: usize,
        language: &Language,
        period: &Period,
    ) {
        self.layers_visited.insert((word_id.to_string(), stage));
        self.languages.insert(language.clone());
        match &self.deepest {
            Some(p) if p.sort_key <= period.sort_key => {}
            _ => self.deepest = Some(period.clone()),
        }
    }

    /// Number of distinct words explored.
    pub fn words_explored(&self) -> usize {
        self.words_explored.len()
    }

    /// Number of distinct historical layers visited.
    pub fn layers_visited(&self) -> usize {
        self.layers_visited.len()
    }

    /// Number of distinct languages encountered.
    pub fn languages_count(&self) -> usize {
        self.languages.len()
    }

    /// The deepest period reached, if any.
    pub fn deepest(&self) -> Option<&Period> {
        self.deepest.as_ref()
    }

    /// Elapsed session time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Session duration formatted as MM:SS (or HH:MM:SS past an hour).
    pub fn elapsed_formatted(&self) -> String {
        let secs = self.elapsed().as_secs();
        let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
        if h > 0 {
            format!("{h:02}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn period(sort_key: i64) -> Period {
        Period {
            label: format!("y{sort_key}"),
            sort_key,
        }
    }

    #[test]
    fn distinct_counting() {
        let mut s = GlobalStats::new();
        s.record_word("salary");
        s.record_word("salary");
        s.record_word("clue");
        assert_eq!(s.words_explored(), 2);

        s.record_layer("salary", 0, &Language::English, &period(2000));
        s.record_layer("salary", 0, &Language::English, &period(2000));
        s.record_layer("salary", 1, &Language::OldFrench, &period(1300));
        assert_eq!(s.layers_visited(), 2);
        assert_eq!(s.languages_count(), 2);
    }

    #[test]
    fn deepest_tracks_minimum_sort_key() {
        let mut s = GlobalStats::new();
        s.record_layer("a", 0, &Language::English, &period(2000));
        s.record_layer("a", 1, &Language::Latin, &period(-100));
        s.record_layer("a", 2, &Language::Greek, &period(500));
        assert_eq!(s.deepest().unwrap().sort_key, -100);
    }

    #[test]
    fn elapsed_format_is_mmss() {
        let s = GlobalStats::new();
        let f = s.elapsed_formatted();
        assert_eq!(f.len(), 5, "expected MM:SS, got {f}");
        assert_eq!(&f[2..3], ":");
    }
}
