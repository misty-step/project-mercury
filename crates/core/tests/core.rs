use std::path::Path;

use project_mercury_core::{load, search, validate, Catalog, Provenance, Query};

fn shipped() -> Catalog {
    load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")).expect("data loads")
}

#[test]
fn shipped_data_validates() {
    let catalog = shipped();
    assert_eq!(catalog.entries.len(), 6);
    assert_eq!(validate(&catalog), Ok(()));
}

#[test]
fn rejects_claim_with_unknown_source() {
    let mut catalog = shipped();
    catalog.entries[0].claims[0].source_id = "nope".into();
    let errors = validate(&catalog).unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("unknown source `nope`")),
        "{errors:?}"
    );
}

#[test]
fn rejects_measured_claim_without_checked_on() {
    let mut catalog = shipped();
    let claim = &mut catalog.entries[0].claims[0];
    claim.provenance = Provenance::Measured;
    claim.checked_on = None;
    let errors = validate(&catalog).unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("no checked_on")),
        "{errors:?}"
    );
}

#[test]
fn rejects_duplicate_ids() {
    let mut catalog = shipped();
    catalog.entries.push(catalog.entries[0].clone());
    assert!(validate(&catalog).unwrap_err()[0].contains("duplicate entry id"));
}

#[test]
fn age_9_math_includes_math_academy() {
    let catalog = shipped();
    let q = Query {
        subject: Some("math".into()),
        age: Some(9),
        ..Query::default()
    };
    let ids: Vec<_> = search(&catalog.entries, &q)
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    assert!(ids.contains(&"math-academy"), "{ids:?}");
    assert!(!ids.contains(&"mentava"), "{ids:?}");
}

#[test]
fn term_and_approach_filters() {
    let catalog = shipped();
    let q = Query {
        term: Some("CATHOLIC".into()),
        ..Query::default()
    };
    let hits = search(&catalog.entries, &q);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "koinonia-hybrid-academy");
    let q = Query {
        approach: Some("ai-tutor".into()),
        ..Query::default()
    };
    assert_eq!(search(&catalog.entries, &q)[0].id, "alpha-school");
}
