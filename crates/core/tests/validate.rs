use mercury_core::{load, search, validate, Catalog, Entry, Source};
use std::path::Path;

const SOURCE: &str = "id: src\ntitle: Example\nurl: https://example.org/\n";

fn catalog(entry_yaml: &str) -> Catalog {
    Catalog {
        entries: vec![serde_yaml::from_str::<Entry>(entry_yaml).unwrap()],
        sources: vec![serde_yaml::from_str::<Source>(SOURCE).unwrap()],
    }
}

fn entry(claims: &str) -> String {
    format!(
        "id: e\nname: Phonics Club\nurl: https://example.org/\nsubjects: [reading]\n\
         ages: {{ min: 4, max: 6 }}\nclaims:\n{claims}"
    )
}

fn messages(c: &Catalog) -> Vec<String> {
    validate(c)
        .unwrap_err()
        .into_iter()
        .map(|p| p.to_string())
        .collect()
}

#[test]
fn accepts_sourced_measured_claim() {
    let c = catalog(&entry(
        "  - { text: Free, source: src, provenance: measured, checked_on: 2026-11-02 }\n",
    ));
    assert_eq!(validate(&c), Ok(()));
}

#[test]
fn rejects_claim_with_no_source() {
    let c = catalog(&entry("  - { text: Free, provenance: inferred }\n"));
    assert_eq!(messages(&c), ["e.claims[0]: claim has no source"]);
}

#[test]
fn rejects_entry_with_no_claims() {
    let c = catalog(&entry("  []\n").replace("claims:\n  []", "claims: []"));
    assert_eq!(messages(&c), ["e: entry has no claims, so no source"]);
}

#[test]
fn rejects_claim_citing_unknown_source() {
    let c = catalog(&entry(
        "  - { text: Free, source: nope, provenance: guess }\n",
    ));
    assert_eq!(
        messages(&c),
        ["e.claims[0]: claim cites unknown source 'nope'"]
    );
}

#[test]
fn rejects_measured_claim_without_checked_on() {
    let c = catalog(&entry(
        "  - { text: Free, source: src, provenance: measured }\n",
    ));
    assert_eq!(
        messages(&c),
        ["e.claims[0]: measured claim has no checked_on"]
    );
}

#[test]
fn rejects_malformed_checked_on() {
    let c = catalog(&entry(
        "  - { text: Free, source: src, provenance: measured, checked_on: 'last week' }\n",
    ));
    assert_eq!(
        messages(&c),
        ["e.claims[0]: checked_on 'last week' is not YYYY-MM-DD"]
    );
}

#[test]
fn rejects_unknown_provenance() {
    let yaml = entry("  - { text: Free, source: src, provenance: vibes }\n");
    assert!(serde_yaml::from_str::<Entry>(&yaml).is_err());
}

#[test]
fn repo_data_is_valid() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let c = load(&dir).unwrap();
    assert_eq!(c.entries.len(), 3);
    assert_eq!(validate(&c), Ok(()));
}

#[test]
fn search_matches_subject_and_age() {
    let c = catalog(&entry(
        "  - { text: Free, source: src, provenance: guess }\n",
    ));
    let hits = search(&c.entries, "reading age 5");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].score, 2);
    assert_eq!(search(&c.entries, "reading age 9")[0].score, 1);
    assert_eq!(search(&c.entries, "phon")[0].entry.id, "e");
    assert!(search(&c.entries, "chemistry 12").is_empty());
    assert!(search(&c.entries, "").is_empty());
}
