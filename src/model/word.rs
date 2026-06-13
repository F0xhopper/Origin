//! Core domain types: words and their reverse etymology chains.

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};

/// The language a historical form belongs to.
///
/// A small known set gets dedicated variants (so the theme can colour them and
/// stats can group them), with an `Other` escape hatch for anything else in the
/// dataset. It (de)serializes as a plain string label, so the dataset just
/// names a language and unknown names land in [`Language::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    MiddleEnglish,
    OldEnglish,
    OldFrench,
    French,
    Latin,
    LateLatin,
    Greek,
    OldNorse,
    Arabic,
    Sanskrit,
    Italian,
    Spanish,
    German,
    Dutch,
    ProtoGermanic,
    ProtoIndoEuropean,
    /// Any language not covered by a dedicated variant.
    Other(String),
}

impl Language {
    /// Map a label string back to a [`Language`], falling back to `Other`.
    pub fn from_label(s: &str) -> Self {
        match s {
            "English" => Language::English,
            "Middle English" => Language::MiddleEnglish,
            "Old English" => Language::OldEnglish,
            "Old French" => Language::OldFrench,
            "French" => Language::French,
            "Latin" => Language::Latin,
            "Late Latin" => Language::LateLatin,
            "Greek" => Language::Greek,
            "Old Norse" => Language::OldNorse,
            "Arabic" => Language::Arabic,
            "Sanskrit" => Language::Sanskrit,
            "Italian" => Language::Italian,
            "Spanish" => Language::Spanish,
            "German" => Language::German,
            "Dutch" => Language::Dutch,
            "Proto-Germanic" => Language::ProtoGermanic,
            "Proto-Indo-European" => Language::ProtoIndoEuropean,
            other => Language::Other(other.to_string()),
        }
    }

    /// Whether this is a reconstructed proto-language (deepest tier).
    pub fn is_proto(&self) -> bool {
        matches!(self, Language::ProtoGermanic | Language::ProtoIndoEuropean)
    }

    /// Human-readable label for display.
    pub fn label(&self) -> &str {
        match self {
            Language::English => "English",
            Language::MiddleEnglish => "Middle English",
            Language::OldEnglish => "Old English",
            Language::OldFrench => "Old French",
            Language::French => "French",
            Language::Latin => "Latin",
            Language::LateLatin => "Late Latin",
            Language::Greek => "Greek",
            Language::OldNorse => "Old Norse",
            Language::Arabic => "Arabic",
            Language::Sanskrit => "Sanskrit",
            Language::Italian => "Italian",
            Language::Spanish => "Spanish",
            Language::German => "German",
            Language::Dutch => "Dutch",
            Language::ProtoGermanic => "Proto-Germanic",
            Language::ProtoIndoEuropean => "Proto-Indo-European",
            Language::Other(name) => name,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s.trim().is_empty() {
            return Err(de::Error::custom("language label must not be empty"));
        }
        Ok(Language::from_label(&s))
    }
}

/// An approximate historical period.
///
/// `sort_key` is a signed year (negative = BCE) used to compare etymological
/// depth across words ("deepest descent reached"). Proto-languages use very
/// large negative sentinels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Period {
    /// Human label, e.g. "c. 1300 CE" or "PIE root".
    pub label: String,
    /// Signed year for ordering; smaller = older / deeper.
    pub sort_key: i64,
}

/// One historical layer in a word's descent.
///
/// In a [`Word::chain`], index 0 is the modern anchor and later indices are
/// progressively older.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EtymologyLayer {
    /// Spelling/form at this stage, e.g. "salarium".
    pub form: String,
    /// Language of origin at this stage.
    pub language: Language,
    /// Approximate era.
    pub period: Period,
    /// Gloss / meaning at this stage.
    pub meaning: String,
    /// Optional cultural or phonetic transition note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// A modern word together with its full reverse etymology chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Word {
    /// Stable slug used as a key for search and sorting.
    pub id: String,
    /// Modern display form, e.g. "Salary".
    pub headword: String,
    /// Anchor gloss shown at the top of the dive.
    pub modern_meaning: String,
    /// Layers ordered modern -> oldest. `chain[0]` is the modern form.
    pub chain: Vec<EtymologyLayer>,
}

impl Word {
    /// The deepest (oldest) layer in the chain.
    pub fn deepest(&self) -> &EtymologyLayer {
        // Invariant (validated at load): chain is non-empty.
        self.chain
            .last()
            .expect("word chain is non-empty by invariant")
    }

    /// Number of historical layers, including the modern anchor.
    pub fn depth(&self) -> usize {
        self.chain.len()
    }

    /// The deepest period reached by this word.
    pub fn deepest_period(&self) -> &Period {
        &self.deepest().period
    }
}
