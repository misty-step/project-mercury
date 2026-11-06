use std::path::Path;

use project_mercury_core::{
    facets, load, search, validate, Catalog, CostTier, Entry, Format, Provenance, Query,
    Regulation, Source, State, STATE_CODES,
};

fn shipped() -> Catalog {
    load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")).expect("data loads")
}

#[test]
fn shipped_data_validates() {
    let catalog = shipped();
    assert_eq!(catalog.entries.len(), 6);
    assert_eq!(validate(&catalog), Ok(()));
    for code in STATE_CODES {
        assert!(catalog.state(code).is_some(), "no record for {code}");
    }
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
        ..Catalog::default()
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

/// A valid two-pathway state: `home` needs only notice, `umbrella` needs approval.
fn fixture() -> Catalog {
    let duty = |req: bool| format!("{{ required: {req}, detail: d }}");
    let pathway = |id: &str, notice: bool, approval: bool| {
        format!(
            "  - id: {id}\n    name: {id}\n    summary: s\n    notice: {}\n    approval: {}\n    \
             assessment: {}\n    subjects: {}\n    instruction_time: {}\n    \
             teacher_qualifications: {}\n    records: {}\n",
            duty(notice),
            duty(approval),
            duty(false),
            duty(false),
            duty(false),
            duty(false),
            duty(false)
        )
    };
    let mut yaml = String::from(
        "code: TN\nname: Tennessee\nsummary: s\ncompulsory_ages: { min: 6, max: 17 }\n\
         funding: { available: false, detail: d }\n\
         public_school_access: { available: true, detail: d }\npathways:\n",
    );
    yaml += &pathway("umbrella", true, true);
    yaml += &pathway("home", true, false);
    yaml += "claims: []\n";
    let mut state: State = serde_yaml::from_str(&yaml).expect("fixture parses");
    for field in state.claim_fields() {
        let claim = format!(
            "{{ field: {field}, value: v, quote: q, provenance: measured, source_id: tn-code, checked_on: '2026-11-05' }}"
        );
        state.claims.push(serde_yaml::from_str(&claim).unwrap());
    }
    let source = Source {
        id: "tn-code".into(),
        url: "https://example.test/tn".into(),
        publisher: "Tennessee General Assembly".into(),
        kind: "statute".into(),
        checked_on: "2026-11-05".into(),
    };
    Catalog {
        states: vec![state],
        sources: vec![source],
        ..Catalog::default()
    }
}

fn state_errors(edit: impl FnOnce(&mut Catalog)) -> Vec<String> {
    let mut c = fixture();
    edit(&mut c);
    validate(&c).err().unwrap_or_default()
}

#[test]
fn state_regulation_is_its_least_regulated_pathway() {
    let mut c = fixture();
    let s = &mut c.states[0];
    assert_eq!(s.pathways[0].regulation(), Regulation::High);
    assert_eq!(s.regulation(), Regulation::Low);
    assert_eq!(s.easiest().unwrap().id, "home");
    s.pathways[1].assessment.required = true;
    assert_eq!(s.regulation(), Regulation::Moderate);
    s.pathways[0].approval.required = false;
    s.pathways[0].notice.required = false;
    assert_eq!(s.regulation(), Regulation::NoNotice);
    assert_eq!(s.easiest().unwrap().id, "umbrella");
    s.pathways[0].teacher_qualifications.required = true;
    assert_eq!(s.regulation(), Regulation::Low);
}

#[test]
fn state_claims_need_quote_primary_source_and_date() {
    assert_eq!(state_errors(|_| {}), Vec::<String>::new());
    let has = |errors: Vec<String>, want: &str| {
        assert!(
            errors.iter().any(|e| e.contains(want)),
            "{want}: {errors:?}"
        );
    };
    has(
        state_errors(|c| c.states[0].claims[0].quote = None),
        "has no quote",
    );
    has(
        state_errors(|c| c.sources[0].kind = "official-site".into()),
        "need statute or agency",
    );
    has(
        state_errors(|c| c.states[0].claims[0].provenance = Provenance::Guess),
        "is a guess",
    );
    has(
        state_errors(|c| c.states[0].claims[0].checked_on = Some("Nov 5".into())),
        "YYYY-MM-DD",
    );
}

#[test]
fn state_fields_need_claims_and_known_names() {
    let has = |errors: Vec<String>, want: &str| {
        assert!(
            errors.iter().any(|e| e.contains(want)),
            "{want}: {errors:?}"
        );
    };
    has(
        state_errors(|c| {
            c.states[0]
                .claims
                .retain(|cl| cl.field != "pathways.home.records")
        }),
        "`pathways.home.records` has no backing claim",
    );
    has(
        state_errors(|c| c.states[0].claims[0].field = "pathways.hmoe.notice".into()),
        "unknown field `pathways.hmoe.notice`",
    );
    has(
        state_errors(|c| c.states[0].code = "XX".into()),
        "not a US state",
    );
    has(
        state_errors(|c| c.states.push(c.states[0].clone())),
        "duplicate state `TN`",
    );
    has(
        state_errors(|c| c.states[0].pathways[1].id = "umbrella".into()),
        "duplicate pathway id",
    );
}
