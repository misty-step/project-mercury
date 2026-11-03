use std::fs;
use std::path::Path;

use mercury_site::{build, primary_source, render};
use project_mercury_core::{load, Catalog};

fn data_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"))
}

fn shipped() -> Catalog {
    load(data_dir()).expect("data loads")
}

fn file<'a>(files: &'a [(String, String)], path: &str) -> &'a str {
    let found = files.iter().find(|(p, _)| p == path);
    &found.unwrap_or_else(|| panic!("missing {path}")).1
}

#[test]
fn index_renders_one_card_per_entry_with_source_best_at_and_age_bar() {
    let catalog = shipped();
    assert_eq!(catalog.entries.len(), 6);
    let files = render(&catalog).unwrap();
    let index = file(&files, "index.html");
    let grid = index
        .split(r#"<ul class="grid" id="grid">"#)
        .nth(1)
        .unwrap();
    let cards: Vec<&str> = grid.split(r#"<li class="card""#).skip(1).collect();
    assert_eq!(cards.len(), catalog.entries.len());
    for (card, e) in cards.iter().zip(&catalog.entries) {
        assert!(
            card.contains(&format!(r#"href="/entries/{}/""#, e.id)),
            "{}",
            e.id
        );
        assert!(card.contains(&e.name), "missing name {}", e.id);
        let src = format!(r#"class="src" href="{}""#, primary_source(&catalog, e));
        assert!(card.contains(&src), "missing source link for {}", e.id);
        assert!(
            card.contains(e.best_at.as_deref().unwrap()),
            "best_at {}",
            e.id
        );
        let span = format!("{}–{}", e.ages.min, e.ages.max);
        assert!(
            card.contains(r#"<div class="age""#) && card.contains(&span),
            "{}",
            e.id
        );
        let filled = card.split(r#"<span class="cells""#).nth(1).unwrap();
        let filled = filled
            .split("</span>")
            .next()
            .unwrap()
            .matches(r#"class="on""#);
        assert_eq!(
            filled.count(),
            usize::from(e.ages.max - e.ages.min + 1),
            "{}",
            e.id
        );
    }
    assert!(index.contains(r#"<select id="age""#));
}

#[test]
fn index_has_header_links_and_no_layout_copy() {
    let files = render(&shipped()).unwrap();
    let index = file(&files, "index.html");
    let repo =
        r#"href="https://github.com/misty-step/project-mercury" aria-label="GitHub repository""#;
    assert!(index.contains(repo));
    assert!(index.contains(">Resources<") && index.contains(">Approaches<"));
    for gone in [
        "Answer first",
        "How to read",
        "Resources to check",
        "Every fact links",
        "Open source",
    ] {
        assert!(
            !index.to_lowercase().contains(&gone.to_lowercase()),
            "still has {gone:?}"
        );
    }
}

#[test]
fn index_renders_facet_chips_with_counts() {
    let catalog = shipped();
    let files = render(&catalog).unwrap();
    let index = file(&files, "index.html");
    let chip = |axis: &str, value: &str| {
        let start = format!(r#"data-axis="{axis}" data-value="{value}""#);
        let rest = index
            .split(&start)
            .nth(1)
            .unwrap_or_else(|| panic!("no chip {axis}={value}"));
        let n = rest.split(r#"<span class="n">"#).nth(1).unwrap();
        n.split('<').next().unwrap().parse::<usize>().unwrap()
    };
    assert_eq!(chip("subject", "math"), 5);
    assert_eq!(chip("format", "online"), 4);
    assert_eq!(chip("format", "hybrid"), 1);
    assert_eq!(chip("cost", "free"), 1);
    assert_eq!(chip("skill", "advanced"), 3);
    assert_eq!(chip("parent", "low"), 4);
    assert_eq!(chip("screen", "medium"), 4);
    // Zero-count values get no chip.
    assert!(!index.contains(r#"data-axis="screen" data-value="high""#));
    let rows = index
        .split(r#"<ol id="cover">"#)
        .nth(1)
        .unwrap()
        .split("</ol>")
        .next()
        .unwrap();
    assert_eq!(rows.matches("data-for=").count(), catalog.entries.len());
}

#[test]
fn every_entry_page_exists_and_links_each_claim_source() {
    let catalog = shipped();
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("site-test");
    let _ = fs::remove_dir_all(&out);
    build(data_dir(), &out).unwrap();
    for name in [
        "index.html",
        "entries/index.html",
        "approaches/index.html",
        "styles.css",
    ] {
        assert!(out.join(name).is_file(), "missing {name}");
    }
    for name in ["404.html", "llms.txt", "dump.json"] {
        assert!(out.join(name).is_file(), "missing {name}");
    }
    let llms = fs::read_to_string(out.join("llms.txt")).unwrap();
    for e in &catalog.entries {
        let page = fs::read_to_string(out.join(format!("entries/{}/index.html", e.id)))
            .unwrap_or_else(|_| panic!("no page for {}", e.id));
        let tbody = page.split("<tbody>").nth(1).unwrap();
        let rows: Vec<&str> = tbody.split("<tr>").skip(1).collect();
        assert_eq!(rows.len(), e.claims.len(), "{}", e.id);
        for (row, claim) in rows.iter().zip(&e.claims) {
            let url = &catalog.source(&claim.source_id).unwrap().url;
            assert!(row.contains(&format!(r#"href="{url}""#)), "{}: {row}", e.id);
        }
        for a in &e.approaches {
            assert!(out.join(format!("approaches/{a}/index.html")).is_file());
        }
        let line = format!("https://projectmercury.mistystep.io/entries/{}/", e.id);
        assert!(llms.contains(&line));
    }
    let dump: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("dump.json")).unwrap()).unwrap();
    assert_eq!(dump["entries"].as_array().unwrap().len(), 6);
}

#[test]
fn text_fields_are_escaped() {
    let mut catalog = shipped();
    let e = &mut catalog.entries[0];
    e.name = "<script>alert(1)</script> Tom & Jerry".into();
    e.summary = r#"Say "hi" & <b>bye</b>"#.into();
    e.best_at = Some("<i>best</i> & 'quoted'".into());
    e.most_useful_when = Some("<u>when</u> & then".into());
    e.claims[0].value = "<script>x()</script> & more".into();
    e.subjects.push("arts & <crafts>".into());
    let id = e.id.clone();
    let files = render(&catalog).unwrap();
    for path in ["index.html".to_string(), format!("entries/{id}/index.html")] {
        let html = file(&files, &path);
        assert!(
            !html.contains("<script>alert") && !html.contains("<script>x()"),
            "{path}"
        );
        assert!(
            !html.contains("<b>bye") && !html.contains("<crafts>") && !html.contains("<i>best"),
            "{path}"
        );
        assert!(
            html.contains("&lt;script&gt;alert(1)&lt;/script&gt; Tom &amp; Jerry")
                && html.contains("&lt;i&gt;best&lt;/i&gt; &amp; &#39;quoted&#39;")
                && html.contains("&lt;u&gt;when&lt;/u&gt; &amp; then")
                && !html.contains("<u>when"),
            "{path}"
        );
    }
    let index = file(&files, "index.html");
    assert!(index.contains(r#"data-value="arts &amp; &lt;crafts&gt;""#));
    assert!(index.contains("arts &amp; &lt;crafts&gt;<span"));
    let entry = file(&files, &format!("entries/{id}/index.html"));
    assert!(entry.contains("&lt;script&gt;x()&lt;/script&gt; &amp; more"));
    assert!(entry.contains("Say &quot;hi&quot; &amp; &lt;b&gt;bye&lt;/b&gt;"));
}

#[test]
fn non_http_urls_are_neutralised() {
    let mut catalog = shipped();
    catalog.entries[0].url = "javascript:alert(1)".into();
    catalog.entries[0].claims.clear();
    let files = render(&catalog).unwrap();
    assert!(!file(&files, "index.html").contains("javascript:"));
}
