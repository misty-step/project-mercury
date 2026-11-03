//! Static site generator for the Project Mercury catalog: plain string
//! templates, every interpolated value HTML-escaped.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use project_mercury_core::{load, validate, Catalog, Entry, Provenance};

pub const BASE_URL: &str = "https://projectmercury.mistystep.io";
const CSS: &str = include_str!("styles.css");
const FILTER_JS: &str = include_str!("filter.js");

/// Escape text for element content and quoted attribute values.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Escaped external href; anything but http(s) becomes `#`.
fn href(url: &str) -> String {
    let ok = url.starts_with("https://") || url.starts_with("http://");
    if ok {
        esc(url)
    } else {
        "#".into()
    }
}

/// Path-safe slug for ids used in URLs.
pub fn slug(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    lower
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// The URL of the first source an entry cites, falling back to its website.
pub fn primary_source<'a>(c: &'a Catalog, e: &'a Entry) -> &'a str {
    let first = e.claims.first().and_then(|cl| c.source(&cl.source_id));
    first.map_or(&e.url, |s| &s.url)
}

/// Load and validate `data`, then write every page under `out`.
/// Returns the number of files written.
pub fn build(data: &Path, out: &Path) -> Result<usize> {
    let catalog = load(data)?;
    if let Err(errors) = validate(&catalog) {
        bail!("invalid catalog:\n{}", errors.join("\n"));
    }
    let files = render(&catalog)?;
    for (rel, body) in &files {
        let path = out.join(rel);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&path, body)?;
    }
    Ok(files.len())
}

/// Every output file as (path relative to the output dir, contents).
pub fn render(c: &Catalog) -> Result<Vec<(String, String)>> {
    let all: Vec<&Entry> = c.entries.iter().collect();
    let approaches: BTreeSet<&String> = c.entries.iter().flat_map(|e| &e.approaches).collect();
    let mut files = vec![
        ("index.html".to_string(), home(c)),
        ("styles.css".into(), CSS.into()),
        ("dump.json".into(), serde_json::to_string_pretty(c)? + "\n"),
        ("llms.txt".into(), llms(c)),
        ("404.html".into(), page("Not found", NOT_FOUND)),
        (
            "entries/index.html".into(),
            listing("All resources", "Catalog", &rows(c, &all)),
        ),
    ];
    for e in &c.entries {
        files.push((
            format!("entries/{}/index.html", slug(&e.id)),
            entry_page(c, e),
        ));
    }
    let mut index = String::from(r#"<ul class="rows">"#);
    for a in &approaches {
        let hits: Vec<&Entry> = all
            .iter()
            .copied()
            .filter(|e| e.approaches.contains(a))
            .collect();
        let (s, n) = (slug(a), hits.len());
        index += &format!(
            r#"<li><a class="name" href="/approaches/{s}/">{}</a><span class="label">{n} {}</span></li>"#,
            esc(a),
            if n == 1 { "resource" } else { "resources" }
        );
        let body = listing(a, "Approach", &rows(c, &hits));
        files.push((format!("approaches/{s}/index.html"), body));
    }
    files.push((
        "approaches/index.html".into(),
        listing("Approaches", "Teaching approaches", &(index + "</ul>")),
    ));
    Ok(files)
}

const NOT_FOUND: &str = r#"<p class="label blue">404</p><h1>Page not found</h1>
<p class="lede">Try the <a href="/">home page</a> or <a href="/entries/">all resources</a>.</p>"#;

fn page(title: &str, body: &str) -> String {
    let title = if title == "Project Mercury" {
        title.to_string()
    } else {
        format!("{title} · Project Mercury")
    };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<link rel="stylesheet" href="/styles.css">
</head>
<body>
<header class="wrap top">
<a class="wordmark" href="/">Project Mercury</a>
<nav><a href="/">Find</a><a href="/approaches/">Approaches</a><a href="/#about">About</a></nav>
<span class="label">Open source · MIT</span>
</header>
<main class="wrap">
{body}
</main>
<footer class="band"><div class="wrap">
<p>Every fact links to its source.</p>
<a class="button" href="/entries/">See all resources →</a>
</div></footer>
</body>
</html>
"#,
        esc(&title)
    )
}

fn listing(title: &str, label: &str, list: &str) -> String {
    let body = format!(
        r#"<p class="label blue">{}</p><h1>{}</h1>{list}"#,
        esc(label),
        esc(title)
    );
    page(title, &body)
}

/// Resource rows; data attributes drive the home page filter.
fn rows(c: &Catalog, entries: &[&Entry]) -> String {
    let mut s = String::from(r#"<ul class="rows">"#);
    for e in entries {
        let subjects: Vec<String> = e.subjects.iter().map(|x| esc(&x.to_lowercase())).collect();
        s += &format!(
            r#"
<li data-subjects="{}" data-age-min="{}" data-age-max="{}"><div><a class="name" href="/entries/{}/">{}</a><p>{}</p></div><a class="src" href="{}">Source: link</a></li>"#,
            subjects.join("|"),
            e.ages.min,
            e.ages.max,
            slug(&e.id),
            esc(&e.name),
            esc(&e.summary),
            href(primary_source(c, e)),
        );
    }
    s + "\n</ul>"
}

fn home(c: &Catalog) -> String {
    let subjects: BTreeSet<String> = c
        .entries
        .iter()
        .flat_map(|e| &e.subjects)
        .map(|s| s.to_lowercase())
        .collect();
    let opt = |v: String| format!("<option>{v}</option>");
    let ages: String = (3..=18).map(|a: u8| opt(a.to_string())).collect();
    let subjects: String = subjects.iter().map(|s| opt(esc(s))).collect();
    let all: Vec<&Entry> = c.entries.iter().collect();
    let body = format!(
        r#"<div class="cols">
<section>
<p class="label blue">Answer first</p>
<h1 class="answer">My child is <span class="nw"><select id="age" aria-label="Child's age"><option value="">any age</option>{ages}</select>,</span> learning <span class="nw"><select id="subject" aria-label="Subject"><option>anything</option>{subjects}</select>.</span></h1>
<div class="card" id="about">
<h2>How to read this catalog</h2>
<p>Each resource is a set of small claims, and each claim says how we know it:</p>
<ul>
<li><span class="badge measured">measured</span> seen on the cited page on the date shown</li>
<li><span class="badge inferred">inferred</span> derived from what the source says, not stated verbatim</li>
<li><span class="badge guess">guess</span> not verified yet</li>
</ul>
<p>Every fact links to its source, so you can check it yourself.</p>
</div>
</section>
<section id="find">
<p class="label">Resources to check · <span id="count">{n}</span></p>
{rows}
<p id="empty" class="empty" hidden>Nothing matches yet. Try "anything" or another age.</p>
</section>
</div>
<script>
{FILTER_JS}</script>"#,
        n = all.len(),
        rows = rows(c, &all),
    );
    page("Project Mercury", &body)
}

fn entry_page(c: &Catalog, e: &Entry) -> String {
    let approaches: Vec<String> = e
        .approaches
        .iter()
        .map(|a| format!(r#"<a href="/approaches/{}/">{}</a>"#, slug(a), esc(a)))
        .collect();
    let mut facts = String::new();
    for cl in &e.claims {
        let src = c.source(&cl.source_id);
        let (url, publisher) = src.map_or(("", &cl.source_id), |s| (s.url.as_str(), &s.publisher));
        let p = match cl.provenance {
            Provenance::Measured => "measured",
            Provenance::Inferred => "inferred",
            Provenance::Guess => "guess",
        };
        facts += &format!(
            r#"
<tr><td class="label">{}</td><td>{}</td><td><span class="badge {p}">{p}</span></td><td><a href="{}">{}</a></td><td class="mono">{}</td></tr>"#,
            esc(&cl.field),
            esc(&cl.value),
            href(url),
            esc(publisher),
            esc(cl.checked_on.as_deref().unwrap_or("—")),
        );
    }
    let body = format!(
        r#"<p class="label blue">{kind}</p>
<h1>{name}</h1>
<p class="lede">{summary}</p>
<dl class="meta">
<dt>Kind</dt><dd>{kind}</dd>
<dt>Ages</dt><dd>{min}–{max}</dd>
<dt>Subjects</dt><dd>{subjects}</dd>
<dt>Approaches</dt><dd>{approaches}</dd>
<dt>Cost</dt><dd>{cost}</dd>
<dt>Website</dt><dd><a href="{url}">{url_text}</a></dd>
</dl>
<h2>Facts</h2>
<table class="facts">
<thead><tr><th>Field</th><th>Value</th><th>Provenance</th><th>Source</th><th>Checked</th></tr></thead>
<tbody>{facts}
</tbody>
</table>"#,
        kind = esc(&e.kind),
        name = esc(&e.name),
        summary = esc(&e.summary),
        min = e.ages.min,
        max = e.ages.max,
        subjects = esc(&e.subjects.join(", ")),
        approaches = approaches.join(", "),
        cost = esc(&e.cost),
        url = href(&e.url),
        url_text = esc(&e.url),
    );
    page(&e.name, &body)
}

fn llms(c: &Catalog) -> String {
    let mut s = format!(
        "# Project Mercury\n\n> A sourced catalog of homeschool resources. Every fact links to its source.\n> Full data: {BASE_URL}/dump.json\n\n"
    );
    for e in &c.entries {
        let summary = e.summary.split_whitespace().collect::<Vec<_>>().join(" ");
        s += &format!(
            "- [{}]({BASE_URL}/entries/{}/): {summary}\n",
            e.name,
            slug(&e.id)
        );
    }
    s
}
