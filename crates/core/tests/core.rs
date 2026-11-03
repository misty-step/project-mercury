use std::path::Path;

use project_mercury_core::{
    facets, load, search, validate, Catalog, CostTier, Entry, Format, Provenance, Query,
};

fn shipped() -> Catalog {
    load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")).expect("data loads")
}

#[test]
fn shipped_data_validates() {
    let catalog = shipped();
    assert_eq!(catalog.entries.len(), 6);
    assert_eq!(validate(&catalog), Ok(()));
    for e in &catalog.entries {
        assert!(e.format.is_some() && e.cost_tier.is_some(), "{}", e.id);
        assert!(
            e.best_at.is_some() && !e.skill_levels.is_empty(),
            "{}",
            e.id
        );
    }
}

#[test]
fn old_entry_without_facets_loads() {
    let yaml = "id: x\nname: X\nurl: https://x.test/\nkind: app\nsummary: s\n\
        subjects: [math]\nages: { min: 5, max: 9 }\napproaches: []\ncost: free\nclaims: []\n";
    let e: Entry = serde_yaml::from_str(yaml).expect("old-style entry loads");
    assert_eq!(
        (e.format, e.cost_tier, e.best_at.as_deref()),
        (None, None, None)
    );
    assert!(e.skill_levels.is_empty());
    let catalog = Catalog {
        entries: vec![e],
        sources: vec![],
    };
    assert_eq!(validate(&catalog), Ok(()));
}

#[test]
fn rejects_facet_without_backing_claim() {
    let mut catalog = shipped();
    catalog.entries[0].claims.retain(|c| c.field != "format");
    let errors = validate(&catalog).unwrap_err();
    assert!(
        errors[0].contains("facet `format` has no backing claim"),
        "{errors:?}"
    );
    let mut catalog = shipped();
    let claim = catalog.entries[0]
        .claims
        .iter_mut()
        .find(|c| c.field == "cost_tier");
    claim.unwrap().source_id = "nope".into();
    assert!(validate(&catalog).unwrap_err()[0].contains("unknown source `nope`"));
}

#[test]
fn filters_by_format_and_cost_tier() {
    let catalog = shipped();
    let ids = |format, cost_tier, skill_level| -> Vec<_> {
        let q = Query {
            format,
            cost_tier,
            skill_level,
            ..Query::default()
        };
        search(&catalog.entries, &q)
            .iter()
            .map(|e| e.id.clone())
            .collect()
    };
    assert_eq!(
        ids(Some(Format::Hybrid), None, None),
        ["koinonia-hybrid-academy"]
    );
    assert_eq!(ids(None, Some(CostTier::Free), None), ["khan-academy"]);
    let advanced = Some("Advanced".parse().unwrap());
    assert_eq!(
        ids(Some(Format::Online), Some(CostTier::Low), advanced),
        ["math-academy"]
    );
}

#[test]
fn facet_counts_sum_to_entry_count() {
    let catalog = shipped();
    let n = catalog.entries.len();
    let all = facets(&catalog.entries);
    assert_eq!(all.len(), 6);
    for f in &all {
        let sum: usize = f.counts.iter().map(|(_, c)| c).sum();
        if f.axis == "skill_levels" {
            let levels = catalog.entries.iter().map(|e| e.skill_levels.len()).sum();
            assert_eq!(sum, levels);
        } else {
            assert_eq!(sum, n, "axis {}", f.axis);
        }
    }
    let format = all.iter().find(|f| f.axis == "format").unwrap();
    assert_eq!(
        format.counts,
        [("online", 4), ("in_person", 1), ("hybrid", 1)]
    );
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
