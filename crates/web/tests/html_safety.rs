use std::sync::LazyLock;

use mercury_site::{esc, render};
use project_mercury_core::{
    Ages, Availability, Catalog, Claim, Duty, Entry, Pathway, Provenance, Source, State,
};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

const TEXT_SLOTS: &[&str] = &[
    "/entries/0/id",
    "/entries/0/name",
    "/entries/0/kind",
    "/entries/0/summary",
    "/entries/0/subjects/0",
    "/entries/0/approaches/0",
    "/entries/0/cost",
    "/entries/0/best_at",
    "/entries/0/most_useful_when",
    "/entries/0/claims/0/field",
    "/entries/0/claims/0/value",
    "/entries/0/claims/0/quote",
    "/entries/0/claims/0/checked_on",
    "/entries/0/claims/0/source_id",
    "/states/0/name",
    "/states/0/summary",
    "/states/0/pathways/0/id",
    "/states/0/pathways/0/name",
    "/states/0/pathways/0/summary",
    "/states/0/pathways/0/notice/detail",
    "/states/0/pathways/0/approval/detail",
    "/states/0/pathways/0/assessment/detail",
    "/states/0/pathways/0/subjects/detail",
    "/states/0/pathways/0/instruction_time/detail",
    "/states/0/pathways/0/teacher_qualifications/detail",
    "/states/0/pathways/0/records/detail",
    "/states/0/funding/detail",
    "/states/0/public_school_access/detail",
    "/states/0/claims/0/field",
    "/states/0/claims/0/value",
    "/states/0/claims/0/quote",
    "/states/0/claims/0/checked_on",
    "/states/0/claims/0/source_id",
    "/sources/0/id",
    "/sources/0/publisher",
];

const ATTACKS: &[&str] = &[
    "",
    "&<>\"'",
    "&lt;script&gt;alert(1)&lt;/script&gt;",
    "&#34; onmouseover=alert(1) x=&#34;",
    "\"><img src=x onerror=alert(1)>",
    "'><svg onload=alert(1)>",
    "</title><script>alert(1)</script>",
    "</script><iframe srcdoc='<script>alert(1)</script>'>",
    "<!-- --><math><mtext></mtext></math>",
    "\0\r\n\t\u{2028}\u{2029}",
    "é日本語👩🏽‍🚀",
];

const URL_PREFIXES: &[&str] = &[
    "",
    "https://example.test/",
    "http://example.test/",
    "https://",
    "http://",
    "javascript:",
    "JaVaScRiPt:",
    "java\tscript:",
    "java\nscript:",
    "data:text/html,",
    "vbscript:",
    "file:///",
    "//example.test/",
    "/example/",
    "#fragment",
    "https:example.test/",
    "HTTP://example.test/",
    "HTTPS://example.test/",
    " https://example.test/",
    "\0javascript:",
    "&#106;avascript:",
];

fn claim(field: impl Into<String>) -> Claim {
    Claim {
        field: field.into(),
        value: "value".into(),
        provenance: Provenance::Measured,
        source_id: "law".into(),
        checked_on: Some("2026-11-05".into()),
        quote: Some("quoted text".into()),
    }
}

fn fixture() -> Catalog {
    let duty = Duty {
        required: true,
        detail: "duty detail".into(),
    };
    let pathway = Pathway {
        id: "home".into(),
        name: "Home education".into(),
        summary: "Path summary".into(),
        notice: duty.clone(),
        approval: Duty {
            required: false,
            ..duty.clone()
        },
        assessment: duty.clone(),
        subjects: duty.clone(),
        instruction_time: duty.clone(),
        teacher_qualifications: duty.clone(),
        records: duty,
    };
    let mut alternative = pathway.clone();
    alternative.id = "alternative".into();
    alternative.approval.required = true;
    let mut state = State {
        code: "TN".into(),
        name: "Tennessee".into(),
        summary: "State summary".into(),
        compulsory_ages: Ages { min: 6, max: 17 },
        pathways: vec![pathway, alternative],
        funding: Availability {
            available: true,
            detail: "Funding detail".into(),
        },
        public_school_access: Availability {
            available: false,
            detail: "Access detail".into(),
        },
        claims: vec![],
    };
    state.claims = state.claim_fields().into_iter().map(claim).collect();
    let entry = Entry {
        id: "resource".into(),
        name: "Resource".into(),
        url: "https://example.test/".into(),
        kind: "book".into(),
        summary: "Resource summary".into(),
        subjects: vec!["topic".into()],
        ages: Ages { min: 6, max: 17 },
        approaches: vec!["approach".into()],
        cost: "Free".into(),
        claims: ["best_at", "most_useful_when"].map(claim).into(),
        best_at: Some("Best at".into()),
        most_useful_when: Some("Useful when".into()),
        skill_levels: vec![],
        format: None,
        cost_tier: None,
        parent_involvement: None,
        screen_time: None,
        pace: None,
    };
    Catalog {
        entries: vec![entry],
        states: vec![state],
        sources: vec![Source {
            id: "law".into(),
            url: "https://example.test/".into(),
            publisher: "State agency".into(),
            kind: "agency".into(),
            checked_on: "2026-11-05".into(),
        }],
    }
}

static FIXTURE: LazyLock<Value> = LazyLock::new(|| serde_json::to_value(fixture()).unwrap());

fn text_catalog(slot: usize, value: &str) -> Catalog {
    let mut data = FIXTURE.clone();
    *data.pointer_mut(TEXT_SLOTS[slot]).unwrap() = Value::String(format!("probe{value}"));
    match TEXT_SLOTS[slot] {
        "/entries/0/summary" => data["entries"][0]["best_at"] = Value::Null,
        "/states/0/claims/0/value" => data["states"][0]["claims"][0]["quote"] = Value::Null,
        _ => {}
    }
    Catalog {
        entries: serde_json::from_value(data["entries"].take()).unwrap(),
        states: serde_json::from_value(data["states"].take()).unwrap(),
        sources: serde_json::from_value(data["sources"].take()).unwrap(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Shape {
    elements: Vec<(usize, String, Vec<String>)>,
    scripts: Vec<String>,
}

fn shape(html: &Html) -> Shape {
    let elements = html
        .root_element()
        .descendants()
        .filter_map(ElementRef::wrap)
        .map(|e| {
            let mut attributes: Vec<_> = e.value().attrs().map(|(k, _)| k.to_string()).collect();
            attributes.sort();
            (
                e.ancestors().count(),
                e.value().name().to_string(),
                attributes,
            )
        })
        .collect();
    let scripts = html
        .select(&Selector::parse("script, style").unwrap())
        .map(|e| e.text().collect())
        .collect();
    Shape { elements, scripts }
}

fn pages(catalog: &Catalog) -> Vec<(String, Html)> {
    render(catalog)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.ends_with(".html"))
        .map(|(path, body)| (path, Html::parse_document(&body)))
        .collect()
}

static TEXT_SHAPES: LazyLock<Vec<Vec<Shape>>> = LazyLock::new(|| {
    (0..TEXT_SLOTS.len())
        .map(|slot| {
            pages(&text_catalog(slot, ""))
                .iter()
                .map(|(_, h)| shape(h))
                .collect()
        })
        .collect()
});

fn assert_text_safe(slot: usize, value: &str) {
    let rendered = pages(&text_catalog(slot, value));
    let baseline = &TEXT_SHAPES[slot];
    assert_eq!(rendered.len(), baseline.len());
    for ((path, html), expected) in rendered.iter().zip(baseline) {
        assert!(
            shape(html) == *expected,
            "HTML structure or executable text changed in {path} through {} with {value:?}",
            TEXT_SLOTS[slot],
        );
    }
}

fn assert_escaped(value: &str) {
    let escaped = esc(value);
    assert!(
        !escaped.contains(['<', '>', '\'', '"']),
        "raw delimiter in {escaped:?}"
    );
    for entity in escaped.split('&').skip(1) {
        assert!(
            ["amp;", "lt;", "gt;", "quot;", "#39;"]
                .iter()
                .any(|e| entity.starts_with(e)),
            "raw ampersand in {escaped:?}",
        );
    }
    let fragment = Html::parse_fragment(&format!(
        "<div data-double=\"{escaped}\" data-single='{escaped}'>{escaped}</div>"
    ));
    let divs: Vec<_> = fragment.select(&Selector::parse("div").unwrap()).collect();
    assert_eq!(divs.len(), 1);
    let div = divs[0];
    assert!(div.children().all(|n| n.value().is_text()));
    assert_eq!(div.value().attrs().count(), 2);
    assert_eq!(div.value().attr("data-double"), Some(value));
    assert_eq!(div.value().attr("data-single"), Some(value));
    assert_eq!(div.text().collect::<String>(), value);
}

fn assert_url_safe(url: &str) {
    let expected = if url.starts_with("http://") || url.starts_with("https://") {
        url.replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\0', "\u{fffd}")
    } else {
        "#".into()
    };
    let links =
        Selector::parse("a.src, .meta dd:last-child > a, table.facts:not(.sl) td a, a.cite")
            .unwrap();
    for fallback in [false, true] {
        let mut catalog = fixture();
        if fallback {
            catalog.entries[0].claims.clear();
        }
        let baseline: Vec<_> = pages(&catalog)
            .iter()
            .map(|(_, html)| shape(html))
            .collect();
        catalog.entries[0].url = url.into();
        catalog.sources[0].url = url.into();
        let rendered = pages(&catalog);
        assert_eq!(rendered.len(), baseline.len());
        let mut checked = 0;
        for ((path, html), original) in rendered.iter().zip(&baseline) {
            assert!(
                shape(html) == *original,
                "URL changed HTML structure in {path}: {url:?}"
            );
            for link in html.select(&links) {
                assert_eq!(
                    link.value().attr("href"),
                    Some(expected.as_str()),
                    "{path}: {url:?}"
                );
                checked += 1;
            }
        }
        assert!(checked > 0, "no external link reached the oracle");
    }
}

#[test]
fn escaped_delimiter_language_is_exhaustive_through_length_four() {
    const ALPHABET: [char; 10] = ['&', '<', '>', '\'', '"', 'l', 't', ';', 'a', ' '];
    for length in 0..=4 {
        for mut index in 0..ALPHABET.len().pow(length) {
            let value: String = (0..length)
                .map(|_| {
                    let c = ALPHABET[index % ALPHABET.len()];
                    index /= ALPHABET.len();
                    c
                })
                .collect();
            assert_escaped(&value);
        }
    }
}

#[test]
fn every_text_sink_rejects_the_attack_corpus() {
    for slot in 0..TEXT_SLOTS.len() {
        for attack in ATTACKS {
            assert_text_safe(slot, attack);
        }
    }
}

#[test]
fn external_links_reject_the_scheme_and_attack_cross_product() {
    for prefix in URL_PREFIXES {
        for attack in ATTACKS {
            assert_url_safe(&format!("{prefix}{attack}"));
        }
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 256,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x4d455243555259),
        failure_persistence: None,
        ..Config::default()
    })]

    #[test]
    fn escaped_unicode_round_trips(
        value in proptest::collection::vec(
            any::<char>().prop_filter("HTML normalizes NUL and CR", |c| *c != '\0' && *c != '\r'),
            0..=64,
        ).prop_map(|chars| chars.into_iter().collect::<String>()),
    ) {
        assert_escaped(&value);
    }

    #[test]
    fn generated_text_cannot_change_html(
        slot in 0..TEXT_SLOTS.len(),
        value in proptest::collection::vec(any::<char>(), 0..=64)
            .prop_map(|chars| chars.into_iter().collect::<String>()),
    ) {
        assert_text_safe(slot, &value);
    }

    #[test]
    fn generated_external_links_are_http_or_neutralized(
        prefix in proptest::sample::select(URL_PREFIXES),
        value in proptest::collection::vec(any::<char>(), 0..=64)
            .prop_map(|chars| chars.into_iter().collect::<String>()),
    ) {
        assert_url_safe(&format!("{prefix}{value}"));
    }
}
