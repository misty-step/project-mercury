//! Project Mercury core: catalog types, YAML loading, validation and search.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A catalog entry: one homeschool resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub kind: String,
    pub summary: String,
    pub subjects: Vec<String>,
    pub ages: Ages,
    pub approaches: Vec<String>,
    /// Free-text cost description.
    pub cost: String,
    pub claims: Vec<Claim>,
}

/// Inclusive age range in years.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ages {
    pub min: u8,
    pub max: u8,
}

/// A sourced statement about one field of an entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub field: String,
    pub value: String,
    pub provenance: Provenance,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_on: Option<String>,
}

/// How confident we are in a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    /// Seen directly on the cited page on `checked_on`.
    Measured,
    /// Derived from what the source says, not stated verbatim.
    Inferred,
    /// Unverified.
    Guess,
}

/// A page that claims cite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub url: String,
    pub publisher: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub checked_on: String,
}

/// Everything loaded from a data directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Catalog {
    pub entries: Vec<Entry>,
    pub sources: Vec<Source>,
}

impl Catalog {
    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn source(&self, id: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.id == id)
    }
}

/// Load `dir/entries/*.yaml` (one entry per file) and `dir/sources/*.yaml`
/// (a list of sources per file). Files are read in name order.
pub fn load(dir: &Path) -> Result<Catalog> {
    let entries = yaml_files(&dir.join("entries"))?
        .iter()
        .map(|p| parse(p))
        .collect::<Result<Vec<Entry>>>()?;
    let mut sources = Vec::new();
    for p in yaml_files(&dir.join("sources"))? {
        sources.extend(parse::<Vec<Source>>(&p)?);
    }
    Ok(Catalog { entries, sources })
}

fn yaml_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml" || x == "yml"))
        .collect();
    files.sort();
    Ok(files)
}

fn parse<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Check referential integrity. Returns every problem found, not just the first.
pub fn validate(catalog: &Catalog) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut source_ids = HashSet::new();
    for s in &catalog.sources {
        if !source_ids.insert(s.id.as_str()) {
            errors.push(format!("duplicate source id `{}`", s.id));
        }
    }
    let mut entry_ids = HashSet::new();
    for e in &catalog.entries {
        if !entry_ids.insert(e.id.as_str()) {
            errors.push(format!("duplicate entry id `{}`", e.id));
        }
        if e.ages.min > e.ages.max {
            errors.push(format!("{}: ages.min > ages.max", e.id));
        }
        for c in &e.claims {
            if !source_ids.contains(c.source_id.as_str()) {
                errors.push(format!(
                    "{}: claim on `{}` cites unknown source `{}`",
                    e.id, c.field, c.source_id
                ));
            }
            if c.provenance == Provenance::Measured && c.checked_on.is_none() {
                errors.push(format!(
                    "{}: measured claim on `{}` has no checked_on",
                    e.id, c.field
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Search filters. Empty fields match everything; text matching is case-insensitive.
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// Substring matched against name, summary and subjects.
    pub term: Option<String>,
    pub subject: Option<String>,
    /// Matches entries with `ages.min <= age <= ages.max`.
    pub age: Option<u8>,
    pub approach: Option<String>,
}

/// Pure filter over entries; preserves input order.
pub fn search<'a>(entries: &'a [Entry], q: &Query) -> Vec<&'a Entry> {
    let eq = |list: &[String], want: &Option<String>| {
        want.as_ref()
            .is_none_or(|w| list.iter().any(|x| x.eq_ignore_ascii_case(w)))
    };
    let term = q.term.as_ref().map(|t| t.to_lowercase());
    entries
        .iter()
        .filter(|e| eq(&e.subjects, &q.subject) && eq(&e.approaches, &q.approach))
        .filter(|e| q.age.is_none_or(|a| (e.ages.min..=e.ages.max).contains(&a)))
        .filter(|e| {
            term.as_ref().is_none_or(|t| {
                [&e.name, &e.summary]
                    .into_iter()
                    .chain(&e.subjects)
                    .any(|s| s.to_lowercase().contains(t))
            })
        })
        .collect()
}
