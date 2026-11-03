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
    // Facets for an at-a-glance catalog. All optional so older data still loads;
    // each one that is set must be backed by a claim whose `field` is its name.
    /// What the resource is best at (about 6 words).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_at: Option<String>,
    /// When it is most useful (about 8 words).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub most_useful_when: Option<String>,
    /// Skill levels the material reaches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_levels: Vec<SkillLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_tier: Option<CostTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_involvement: Option<Level>,
    /// low: up to ~30 min/day; medium: ~30 min to 2 h; high: over 2 h.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_time: Option<Level>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<Pace>,
}

/// A closed set of facet values with stable snake_case names.
pub trait FacetValue: Copy + PartialEq + 'static {
    const ALL: &'static [Self];
    fn as_str(self) -> &'static str;
}

macro_rules! facet_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $var:ident = $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name { $($(#[$vm])* #[serde(rename = $s)] $var),+ }
        impl FacetValue for $name {
            const ALL: &'static [Self] = &[$(Self::$var),+];
            fn as_str(self) -> &'static str { match self { $(Self::$var => $s),+ } }
        }
        /// Case-insensitive; `-` is accepted for `_`.
        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, String> {
                let s = s.trim().to_ascii_lowercase().replace('-', "_");
                Self::ALL.iter().copied().find(|v| v.as_str() == s).ok_or_else(|| {
                    let all: Vec<_> = Self::ALL.iter().map(|v| v.as_str()).collect();
                    format!("expected one of: {}", all.join(", "))
                })
            }
        }
    };
}

facet_enum!(SkillLevel { Beginner = "beginner", Intermediate = "intermediate", Advanced = "advanced" });
facet_enum!(Format { Online = "online", InPerson = "in_person", Hybrid = "hybrid" });
facet_enum!(
    /// Monthly cost: low is under $50, mid $50-300, high over $300.
    CostTier { Free = "free", Low = "low", Mid = "mid", High = "high" }
);
facet_enum!(Level { Low = "low", Medium = "medium", High = "high" });
facet_enum!(Pace { SelfPaced = "self_paced", Scheduled = "scheduled" });

impl Entry {
    /// Facet field names paired with whether this entry sets them.
    fn facets_set(&self) -> [(&'static str, bool); 8] {
        [
            ("best_at", self.best_at.is_some()),
            ("most_useful_when", self.most_useful_when.is_some()),
            ("skill_levels", !self.skill_levels.is_empty()),
            ("format", self.format.is_some()),
            ("cost_tier", self.cost_tier.is_some()),
            ("parent_involvement", self.parent_involvement.is_some()),
            ("screen_time", self.screen_time.is_some()),
            ("pace", self.pace.is_some()),
        ]
    }
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
        for (field, set) in e.facets_set() {
            if set && !e.claims.iter().any(|c| c.field == field) {
                errors.push(format!("{}: facet `{field}` has no backing claim", e.id));
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
    /// Matches entries whose `skill_levels` include this level.
    pub skill_level: Option<SkillLevel>,
    pub format: Option<Format>,
    pub cost_tier: Option<CostTier>,
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
        .filter(|e| q.skill_level.is_none_or(|l| e.skill_levels.contains(&l)))
        .filter(|e| q.format.is_none_or(|f| e.format == Some(f)))
        .filter(|e| q.cost_tier.is_none_or(|c| e.cost_tier == Some(c)))
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

/// Entry counts for every value of one facet axis, in declaration order (zeros included).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FacetCounts {
    /// The `Entry` field name, e.g. `format`.
    pub axis: &'static str,
    pub counts: Vec<(&'static str, usize)>,
}

/// Counts per value for each facet axis, for charts. Single-valued axes sum to the
/// number of entries that set them; `skill_levels` counts each level an entry lists.
pub fn facets(entries: &[Entry]) -> Vec<FacetCounts> {
    fn tally<T: FacetValue>(
        axis: &'static str,
        entries: &[Entry],
        has: impl Fn(&Entry, T) -> bool,
    ) -> FacetCounts {
        let counts = T::ALL
            .iter()
            .map(|&v| (v.as_str(), entries.iter().filter(|e| has(e, v)).count()))
            .collect();
        FacetCounts { axis, counts }
    }
    vec![
        tally("skill_levels", entries, |e, v| e.skill_levels.contains(&v)),
        tally("format", entries, |e, v| e.format == Some(v)),
        tally("cost_tier", entries, |e, v| e.cost_tier == Some(v)),
        tally("parent_involvement", entries, |e, v| {
            e.parent_involvement == Some(v)
        }),
        tally("screen_time", entries, |e, v| e.screen_time == Some(v)),
        tally("pace", entries, |e, v| e.pace == Some(v)),
    ]
}
