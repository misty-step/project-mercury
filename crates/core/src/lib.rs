//! Core types for Project Mercury: entries, claims and sources, plus
//! loading from YAML, validation, and a pure term-match search.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

/// How a claim was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    /// Checked directly against the source on a known date.
    Measured,
    /// Derived from the source but not stated there verbatim.
    Inferred,
    /// Best estimate; treat with caution.
    Guess,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub text: String,
    /// Id of a [`Source`]. Optional at parse time so validation can report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub provenance: Provenance,
    /// ISO date (YYYY-MM-DD) the claim was last checked. Required for measured claims.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_on: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeRange {
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub subjects: Vec<String>,
    pub ages: AgeRange,
    pub claims: Vec<Claim>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    pub entries: Vec<Entry>,
    pub sources: Vec<Source>,
}

impl Catalog {
    pub fn source(&self, id: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.id == id)
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::path::PathBuf, std::io::Error),
    Yaml(std::path::PathBuf, serde_yaml::Error),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(p, e) => write!(f, "{}: {e}", p.display()),
            LoadError::Yaml(p, e) => write!(f, "{}: {e}", p.display()),
        }
    }
}

impl std::error::Error for LoadError {}

/// Load `<dir>/entries/*.yaml` and `<dir>/sources/*.yaml`, one record per file,
/// in file-name order.
pub fn load(dir: &Path) -> Result<Catalog, LoadError> {
    Ok(Catalog {
        entries: load_dir(&dir.join("entries"))?,
        sources: load_dir(&dir.join("sources"))?,
    })
}

fn load_dir<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, LoadError> {
    let io = |e| LoadError::Io(dir.to_path_buf(), e);
    let mut paths = Vec::new();
    for item in std::fs::read_dir(dir).map_err(io)? {
        let path = item.map_err(io)?.path();
        if path.extension().is_some_and(|x| x == "yaml" || x == "yml") {
            paths.push(path);
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).map_err(|e| LoadError::Io(p.clone(), e))?;
            serde_yaml::from_str(&text).map_err(|e| LoadError::Yaml(p, e))
        })
        .collect()
}

/// A single validation failure, tied to the record it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub record: String,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.record, self.message)
    }
}

/// Check that every entry is sourced and every claim is well-formed.
pub fn validate(catalog: &Catalog) -> Result<(), Vec<Problem>> {
    let mut problems = Vec::new();
    let mut push = |record: &str, message: String| {
        problems.push(Problem {
            record: record.to_string(),
            message,
        })
    };

    let mut source_ids = HashSet::new();
    for s in &catalog.sources {
        if !source_ids.insert(s.id.as_str()) {
            push(&s.id, "duplicate source id".into());
        }
    }

    let mut entry_ids = HashSet::new();
    for e in &catalog.entries {
        if !entry_ids.insert(e.id.as_str()) {
            push(&e.id, "duplicate entry id".into());
        }
        if e.ages.min > e.ages.max {
            push(
                &e.id,
                format!("ages.min {} > ages.max {}", e.ages.min, e.ages.max),
            );
        }
        if e.claims.is_empty() {
            push(&e.id, "entry has no claims, so no source".into());
        }
        for (i, c) in e.claims.iter().enumerate() {
            let at = format!("{}.claims[{i}]", e.id);
            match c.source.as_deref().map(str::trim) {
                None | Some("") => push(&at, "claim has no source".into()),
                Some(id) if !source_ids.contains(id) => {
                    push(&at, format!("claim cites unknown source '{id}'"))
                }
                Some(_) => {}
            }
            match c.checked_on.as_deref() {
                None if c.provenance == Provenance::Measured => {
                    push(&at, "measured claim has no checked_on".into())
                }
                Some(d) if !is_iso_date(d) => {
                    push(&at, format!("checked_on '{d}' is not YYYY-MM-DD"))
                }
                _ => {}
            }
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && matches!(s[5..7].parse::<u8>(), Ok(1..=12))
        && matches!(s[8..10].parse::<u8>(), Ok(1..=31))
}

/// A search result: the entry and how many query terms it matched.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit<'a> {
    pub score: usize,
    pub entry: &'a Entry,
}

/// Words that carry no meaning on their own in queries like "reading age 5".
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "age", "aged", "ages", "for", "of", "old", "the", "year", "years",
];

fn tokens(s: &str) -> impl Iterator<Item = String> + '_ {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_lowercase)
}

/// Pure term-match search over name, subjects and ages.
///
/// Numeric terms match entries whose age range contains the number; other
/// terms match a word in the name or subjects exactly, or as a prefix when
/// the term is at least three characters. Results are ranked by the number
/// of matched terms, then by name. Entries matching nothing are omitted.
pub fn search<'a>(entries: &'a [Entry], query: &str) -> Vec<Hit<'a>> {
    let terms: Vec<String> = tokens(query)
        .filter(|t| !STOPWORDS.contains(&t.as_str()))
        .collect();
    let mut hits: Vec<Hit> = entries
        .iter()
        .filter_map(|entry| {
            let words: Vec<String> = tokens(&entry.name)
                .chain(entry.subjects.iter().flat_map(|s| tokens(s)))
                .collect();
            let score = terms
                .iter()
                .filter(|t| match t.parse::<u16>() {
                    Ok(n) => (u16::from(entry.ages.min)..=u16::from(entry.ages.max)).contains(&n),
                    Err(_) => words
                        .iter()
                        .any(|w| w == *t || (t.len() >= 3 && w.starts_with(t.as_str()))),
                })
                .count();
            (score > 0).then_some(Hit { score, entry })
        })
        .collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.entry.name.cmp(&b.entry.name)));
    hits
}
