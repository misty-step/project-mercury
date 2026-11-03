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
fn index_lists_every_entry_with_its_source_link() {
    let catalog = shipped();
    assert_eq!(catalog.entries.len(), 6);
    let files = render(&catalog).unwrap();
    let index = file(&files, "index.html");
    for e in &catalog.entries {
        assert!(index.contains(&e.name), "missing name {}", e.name);
        let link = format!(r#"<a class="src" href="{}">"#, primary_source(&catalog, e));
        assert!(index.contains(&link), "missing source link for {}", e.id);
        assert!(index.contains(&format!(r#"href="/entries/{}/""#, e.id)));
    }
    assert!(index.contains(r#"<select id="age""#) && index.contains(r#"<select id="subject""#));
    assert!(index.contains(r#"href="/entries/">See all resources"#));
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
            !html.contains("<b>bye") && !html.contains("<crafts>"),
            "{path}"
        );
        assert!(
            html.contains("&lt;script&gt;alert(1)&lt;/script&gt; Tom &amp; Jerry"),
            "{path}"
        );
    }
    let index = file(&files, "index.html");
    assert!(index.contains("Say &quot;hi&quot; &amp; &lt;b&gt;bye&lt;/b&gt;"));
    assert!(index.contains("<option>arts &amp; &lt;crafts&gt;</option>"));
    let entry = file(&files, &format!("entries/{id}/index.html"));
    assert!(entry.contains("&lt;script&gt;x()&lt;/script&gt; &amp; more"));
}

#[test]
fn non_http_urls_are_neutralised() {
    let mut catalog = shipped();
    catalog.entries[0].url = "javascript:alert(1)".into();
    catalog.entries[0].claims.clear();
    let files = render(&catalog).unwrap();
    assert!(!file(&files, "index.html").contains("javascript:"));
}
