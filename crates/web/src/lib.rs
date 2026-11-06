//! Static site generator for the Project Mercury catalog: plain string
//! templates, every interpolated value HTML-escaped.

mod states;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use project_mercury_core::{
    facets, load, search, validate, Catalog, Claim, Entry, FacetValue, Format, Level, Provenance,
    Query, SkillLevel,
};

pub const BASE_URL: &str = "https://projectmercury.mistystep.io";
pub const REPO_URL: &str = "https://github.com/misty-step/project-mercury";
const CSS: &str = include_str!("styles.css");
const FILTER_JS: &str = include_str!("filter.js");
/// Ages covered by the age bars and the coverage chart.
const AGES: std::ops::RangeInclusive<u8> = 3..=18;
/// Filter axes: (data attribute, legend, icon, core facet axis).
const AXES: [(&str, &str, &str, &str); 5] = [
    ("skill", "Skill", "bars", "skill_levels"),
    ("format", "Format", "monitor", "format"),
    ("cost", "Cost", "coin", "cost_tier"),
    ("parent", "Parent", "users", "parent_involvement"),
    ("screen", "Screen", "phone", "screen_time"),
];

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
    let resources = listing(c, "Resources", "", &all);
    let mut files = vec![
        ("index.html".to_string(), home(c)),
        ("styles.css".into(), CSS.into()),
        ("dump.json".into(), serde_json::to_string_pretty(c)? + "\n"),
        ("llms.txt".into(), llms(c)),
        ("404.html".into(), page("Not found", "", NOT_FOUND)),
        ("entries/index.html".into(), resources),
    ];
    for e in &c.entries {
        let path = format!("entries/{}/index.html", slug(&e.id));
        files.push((path, entry_page(c, e)));
    }
    let mut index = String::from(r#"<h1>Approaches</h1><ul class="list">"#);
    let crumb = r#"<a class="crumb" href="/approaches/">Approaches</a>"#;
    let approaches: BTreeSet<&String> = c.entries.iter().flat_map(|e| &e.approaches).collect();
    for a in approaches {
        let by_approach = Query {
            approach: Some(a.clone()),
            ..Query::default()
        };
        let hits = search(&c.entries, &by_approach);
        let (s, name, n) = (slug(a), esc(a), hits.len());
        index +=
            &format!(r#"<li><a href="/approaches/{s}/">{name}<span class="n">{n}</span></a></li>"#);
        let path = format!("approaches/{s}/index.html");
        files.push((path, listing(c, a, crumb, &hits)));
    }
    let index = page("Approaches", "/approaches/", &(index + "</ul>"));
    files.push(("approaches/index.html".into(), index));
    files.extend(states::pages(c));
    Ok(files)
}

const NOT_FOUND: &str = r#"<h1>Page not found</h1>
<p class="lede"><a href="/">Back to resources</a></p>"#;

/// The shared shell; `here` is the nav path to mark as current.
fn page(title: &str, here: &str, body: &str) -> String {
    let title = match title {
        "Project Mercury" => title.to_string(),
        t => format!("{} · Project Mercury", esc(t)),
    };
    let mut nav = String::new();
    for (u, t) in [
        ("/", "Resources"),
        ("/approaches/", "Approaches"),
        ("/states/", "States"),
    ] {
        let cur = ["", r#" aria-current="page""#][(u == here) as usize];
        nav += &format!(r#"<a href="{u}"{cur}>{t}</a>"#);
    }
    let gh = icon("github");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<title>{title}</title>
<link rel="stylesheet" href="/styles.css">
<noscript><style>.js-only{{display:none!important}}</style></noscript>
</head>
<body>
<header><div class="wrap bar">
<a class="wordmark" href="/"><svg class="logo" viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="7"/></svg>Project Mercury</a>
<nav aria-label="Main">{nav}</nav>
<a class="ghl" href="{REPO_URL}" aria-label="GitHub repository">{gh}</a>
</div></header>
<main class="wrap">
{body}
</main>
<footer><div class="wrap foot"><a href="/dump.json">dump.json</a><a href="/llms.txt">llms.txt</a></div></footer>
</body>
</html>
"#
    )
}

/// 24px-grid line icons, one `name paths` per line (subjects by name); `github` is filled.
const ICONS: &str = r#"monitor <rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>
check <path d="M20 6 9 17l-5-5"/>
minus <path d="M5 12h14"/>
building <rect x="4" y="2" width="16" height="20" rx="2"/><path d="M9 22v-4h6v4M8 6h.01M12 6h.01M16 6h.01M8 10h.01M12 10h.01M16 10h.01M8 14h.01M12 14h.01M16 14h.01"/>
target <circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1"/>
users <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>
phone <rect x="6" y="2" width="12" height="20" rx="2"/><path d="M11 18h2"/>
coin <circle cx="12" cy="12" r="9"/><path d="M15 9.5c-.5-1-1.6-1.5-3-1.5-1.7 0-3 .8-3 2s1.3 1.7 3 2 3 .8 3 2-1.3 2-3 2c-1.4 0-2.5-.5-3-1.5M12 6v2M12 16v2"/>
bars <path d="M5 20v-5M12 20V10M19 20V4"/>
cake <path d="M4 21v-8a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v8M2 21h20M4 16c2 1 4 1 5.3 0s3.3-1 5.4 0 3.3 1 5.3 0M12 11V8M12 4.5V5"/>
reading <path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2zM22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z"/>
phonics <path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2zM22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z"/>
math <path d="M18 7V4H6l6 8-6 8h12v-3"/>
writing <path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5z"/>
science <path d="M9 3h6M10 3v6L4.6 19a1.5 1.5 0 0 0 1.3 2h12.2a1.5 1.5 0 0 0 1.3-2L14 9V3M7 15h10"/>
external <path d="M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
github <path d="M12 .5a11.5 11.5 0 0 0-3.64 22.42c.58.1.79-.25.79-.56v-2c-3.2.7-3.87-1.37-3.87-1.37-.52-1.33-1.28-1.68-1.28-1.68-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.19 1.76 1.19 1.03 1.76 2.69 1.25 3.35.96.1-.75.4-1.25.73-1.54-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.28 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.17 1.18a11 11 0 0 1 5.77 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.83 1.19 3.09 0 4.42-2.69 5.39-5.26 5.68.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .5z"/>"#;

/// An inline icon; decorative, so aria-hidden. Unknown names render nothing.
fn icon(name: &str) -> String {
    let found = ICONS
        .lines()
        .find_map(|l| l.strip_prefix(name)?.strip_prefix(' '));
    let Some(d) = found else { return String::new() };
    let class = if name == "github" { "i gh" } else { "i" };
    format!(r#"<svg class="{class}" viewBox="0 0 24 24" aria-hidden="true">{d}</svg>"#)
}

/// `<i>` marks with `class="on"` where set; CSS draws them as year cells, dots or pips.
fn marks(on: impl Iterator<Item = bool>) -> String {
    on.map(|b| ["<i></i>", r#"<i class="on"></i>"#][b as usize])
        .collect()
}

/// Facet values an entry has on one core axis, as snake_case strings.
fn values(e: &Entry, axis: &str) -> Vec<&'static str> {
    let level = |l: Option<Level>| l.map(Level::as_str).into_iter().collect();
    match axis {
        "skill_levels" => e.skill_levels.iter().map(|v| v.as_str()).collect(),
        "format" => e.format.map(Format::as_str).into_iter().collect(),
        "cost_tier" => e.cost_tier.map(|v| v.as_str()).into_iter().collect(),
        "parent_involvement" => level(e.parent_involvement),
        "screen_time" => level(e.screen_time),
        _ => vec![],
    }
}

/// Display word for a facet value, plus a longer note for cost tiers.
fn word(axis: &str, v: &str) -> (String, &'static str) {
    let cap = |v: &str| v[..1].to_uppercase() + &v[1..].replace('_', " ");
    match (axis, v) {
        ("cost_tier", "free") => ("Free".into(), ""),
        ("cost_tier", "low") => ("$".into(), "under $50/mo"),
        ("cost_tier", "mid") => ("$$".into(), "$50–300/mo"),
        ("cost_tier", "high") => ("$$$".into(), "over $300/mo"),
        _ => (cap(v), ""),
    }
}

fn cost(e: &Entry) -> String {
    let Some(t) = e.cost_tier else {
        return String::new();
    };
    let (glyph, note) = word("cost_tier", t.as_str());
    format!(r#"<span class="cost" title="{note}">{glyph}<span class="sr"> {note}</span></span>"#)
}

fn format_badge(e: &Entry) -> String {
    let (icons, f) = match e.format {
        Some(Format::Online) => (icon("monitor"), "Online"),
        Some(Format::InPerson) => (icon("building"), "In person"),
        Some(Format::Hybrid) => (icon("monitor") + &icon("building"), "Hybrid"),
        None => return String::new(),
    };
    format!(r#"<span class="fmt">{icons}{f}</span>"#)
}

/// Year cells for ages 3-18, filled where covered, then the span as text.
fn span(e: &Entry) -> String {
    let (a, b) = (e.ages.min, e.ages.max);
    let on = marks(AGES.map(|y| (a..=b).contains(&y)));
    format!(
        r#"<span class="cells" aria-hidden="true">{on}</span><span class="al"><span class="sr">Ages </span>{a}–{b}</span>"#
    )
}

/// Parent-involvement and screen-time dots plus skill pips; `show` makes labels visible.
fn meters(e: &Entry, show: bool) -> String {
    let cls = ["sr", "ml"][show as usize];
    let m = |t: String, ic: &str, kind: &str, on: String| {
        let ic = icon(ic);
        format!(
            r#"<span class="m" title="{t}">{ic}<span class="{kind}" aria-hidden="true">{on}</span><span class="{cls}">{t}</span></span>"#
        )
    };
    let mut s = String::new();
    for (name, ic, lv) in [
        ("Parent involvement", "users", e.parent_involvement),
        ("Screen time", "phone", e.screen_time),
    ] {
        let Some(lv) = lv else { continue };
        let n = Level::ALL.iter().position(|&x| x == lv).unwrap_or(0);
        let dots = marks((0..3).map(|i| i <= n));
        s += &m(format!("{name}: {}", lv.as_str()), ic, "dots", dots);
    }
    if !e.skill_levels.is_empty() {
        let t = format!("Skill: {}", values(e, "skill_levels").join(", "));
        let pips = marks(SkillLevel::ALL.iter().map(|l| e.skill_levels.contains(l)));
        s += &m(t, "", "pips", pips);
    }
    s
}

fn when(e: &Entry) -> String {
    let (i, w) = (icon("target"), e.most_useful_when.as_deref().map(esc));
    let w = w.map(|w| format!(r#"<span class="sr">Most useful when: </span>{w}"#));
    w.map(|w| format!(r#"<p class="when">{i}<span>{w}</span></p>"#))
        .unwrap_or_default()
}

fn tags(e: &Entry) -> String {
    let mut li = String::new();
    for s in e.subjects.iter().map(|s| s.to_lowercase()) {
        li += &format!("<li>{}{}</li>", icon(&s), esc(&s));
    }
    format!(r#"<ul class="tags" aria-label="Subjects">{li}</ul>"#)
}

/// One resource card; data attributes drive the home page filter.
fn card(c: &Catalog, e: &Entry) -> String {
    let subjects: Vec<String> = e.subjects.iter().map(|s| esc(&s.to_lowercase())).collect();
    let mut data = format!(r#"data-subject="{}""#, subjects.join("|"));
    for (k, _, _, axis) in AXES {
        data += &format!(r#" data-{k}="{}""#, values(e, axis).join("|"));
    }
    let (id, name, src) = (slug(&e.id), esc(&e.name), href(primary_source(c, e)));
    let (best, top) = (
        esc(e.best_at.as_deref().unwrap_or(&e.summary)),
        format_badge(e) + &cost(e),
    );
    let (min, max, ext, rest) = (e.ages.min, e.ages.max, icon("external"), when(e) + &tags(e));
    let (span, meters) = (span(e), meters(e, false));
    format!(
        r#"
<li class="card" id="r-{id}" {data} data-age-min="{min}" data-age-max="{max}">
<div class="row">{top}<a class="src" href="{src}" title="Source" aria-label="Source for {name}">{ext}</a></div>
<h3><a href="/entries/{id}/">{name}</a></h3>
<p class="best">{best}</p>
{rest}
<div class="age" title="Ages {min}–{max}">{span}</div>
<div class="row meters">{meters}</div>
</li>"#
    )
}

fn grid(c: &Catalog, entries: &[&Entry]) -> String {
    let cards: String = entries.iter().map(|e| card(c, e)).collect();
    format!("<ul class=\"grid\" id=\"grid\">{cards}\n</ul>")
}

fn listing(c: &Catalog, title: &str, crumb: &str, entries: &[&Entry]) -> String {
    let (t, n, grid) = (esc(title), entries.len(), grid(c, entries));
    let head = format!(r#"<div class="head"><h1>{t}</h1><span class="count">{n}</span></div>"#);
    let here = if crumb.is_empty() {
        "/"
    } else {
        "/approaches/"
    };
    page(title, here, &format!("{crumb}{head}{grid}"))
}

/// A toggle chip; `note` is a tooltip and screen-reader suffix.
fn chip(axis: &str, value: &str, text: &str, note: &str, n: usize) -> String {
    let v = esc(value);
    format!(
        r#"<button type="button" class="chip" data-axis="{axis}" data-value="{v}" aria-pressed="false" title="{note}">{text}<span class="sr"> {note}</span><span class="n">{n}</span></button>"#
    )
}

fn group(id: &str, legend: &str, ic: &str, controls: &str) -> String {
    let i = icon(ic);
    format!(
        r#"<div class="fg" role="group" aria-labelledby="lg-{id}"><span class="lg" id="lg-{id}">{i}{legend}</span><span class="chips">{controls}</span></div>"#
    )
}

/// The filter bar: an age select, then chips with counts for subjects and each facet axis.
fn filters(c: &Catalog) -> String {
    let opts: String = AGES.map(|a| format!("<option>{a}</option>")).collect();
    let select = format!(
        r#"<select id="age" aria-labelledby="lg-age"><option value="">Any</option>{opts}</select>"#
    );
    let mut out = group("age", "Age", "cake", &select);
    let mut subjects: BTreeMap<String, usize> = BTreeMap::new();
    for s in c.entries.iter().flat_map(|e| &e.subjects) {
        *subjects.entry(s.to_lowercase()).or_default() += 1;
    }
    let mut subjects: Vec<_> = subjects.into_iter().collect();
    subjects.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let chip_for = |(s, n): &(String, usize)| chip("subject", s, &(icon(s) + &esc(s)), "", *n);
    let chips: String = subjects.iter().map(chip_for).collect();
    out += &group("subject", "Subject", "reading", &chips);
    let counts = facets(&c.entries);
    for (key, legend, ic, axis) in AXES {
        let Some(f) = counts.iter().find(|f| f.axis == axis) else {
            continue;
        };
        let mut chips = String::new();
        for (v, n) in f.counts.iter().filter(|(_, n)| *n > 0) {
            let (text, note) = word(axis, v);
            chips += &chip(key, v, &text, note, *n);
        }
        out += &group(key, legend, ic, &chips);
    }
    out
}

/// One row per resource across ages 3-18, so gaps show at a glance.
fn coverage(c: &Catalog) -> String {
    let axis: String = AGES.map(|a| format!("<span>{a}</span>")).collect();
    let mut rows = format!(
        r#"<li class="axis" aria-hidden="true"><span></span><span class="cells">{axis}</span></li>"#
    );
    for e in &c.entries {
        let (id, name, span) = (slug(&e.id), esc(&e.name), span(e));
        rows += &format!(
            r#"<li data-for="r-{id}" title="{name}"><span class="cn">{name}</span>{span}</li>"#
        );
    }
    format!(
        r#"<figure class="panel cover"><figcaption class="lg">Age coverage</figcaption><ol id="cover">{rows}</ol></figure>"#
    )
}

fn home(c: &Catalog) -> String {
    let all: Vec<&Entry> = c.entries.iter().collect();
    let (n, filters, coverage, grid) = (all.len(), filters(c), coverage(c), grid(c, &all));
    let body = format!(
        r#"<div class="head"><h1>Resources</h1><span class="count" id="count" aria-live="polite">{n}</span><button type="button" class="btn js-only" id="clear" hidden>Clear</button></div>
<div class="top">
<form class="panel facets js-only" aria-label="Filters">{filters}</form>
{coverage}
</div>
{grid}
<p id="empty" class="empty" hidden>No matches</p>
<script>
{FILTER_JS}</script>"#
    );
    page("Project Mercury", "/", &body)
}

/// The sourced facts table; `label` names each claim's field. Quotes show under values.
fn facts(c: &Catalog, claims: &[Claim], label: impl Fn(&str) -> String) -> String {
    let mut rows = String::new();
    for cl in claims {
        let src = c.source(&cl.source_id);
        let (url, publisher) = src.map_or(("", &cl.source_id), |s| (s.url.as_str(), &s.publisher));
        let (p, tip) = match cl.provenance {
            Provenance::Measured => ("measured", "Seen on the cited page on the date checked"),
            Provenance::Inferred => ("inferred", "Derived from the source, not stated verbatim"),
            Provenance::Guess => ("guess", "Not verified"),
        };
        let (field, value, url, publisher) = (
            esc(&label(&cl.field)),
            esc(&cl.value),
            href(url),
            esc(publisher),
        );
        let quote = cl.quote.as_deref().map(esc);
        let quote = quote
            .map(|q| format!(r#"<q class="quote">{q}</q>"#))
            .unwrap_or_default();
        let checked = esc(cl.checked_on.as_deref().unwrap_or("—"));
        rows += &format!(
            r#"
<tr><td class="mono">{field}</td><td>{value}{quote}</td><td><span class="badge {p}" title="{tip}">{p}</span></td><td><a href="{url}">{publisher}</a></td><td class="mono">{checked}</td></tr>"#
        );
    }
    format!(
        r#"<table class="facts">
<thead><tr><th>Field</th><th>Value</th><th>Provenance</th><th>Source</th><th>Checked</th></tr></thead>
<tbody>{rows}
</tbody>
</table>"#
    )
}

fn entry_page(c: &Catalog, e: &Entry) -> String {
    let facts = facts(c, &e.claims, str::to_string);
    let link = |a: &String| format!(r#"<a href="/approaches/{}/">{}</a>"#, slug(a), esc(a));
    let approaches = e.approaches.iter().map(link).collect::<Vec<_>>().join(", ");
    let best = e.best_at.as_deref().map(esc);
    let best = best
        .map(|b| format!(r#"<p class="lede">{b}</p>"#))
        .unwrap_or_default();
    let (top, kind, name, when) = (format_badge(e), esc(&e.kind), esc(&e.name), when(e));
    let (cost, span, meters, tags) = (cost(e), span(e), meters(e, true), tags(e));
    let (summary, cost_text, url, url_text) =
        (esc(&e.summary), esc(&e.cost), href(&e.url), esc(&e.url));
    let body = format!(
        r#"<div class="hero">
<div class="row">{top}<span class="kind">{kind}</span>{cost}</div>
<h1>{name}</h1>
{best}{when}
<div class="glance"><div class="age">{span}</div>{meters}</div>
</div>
<p class="summary">{summary}</p>
<dl class="meta">
<dt>Subjects</dt><dd>{tags}</dd>
<dt>Approaches</dt><dd>{approaches}</dd>
<dt>Cost</dt><dd>{cost_text}</dd>
<dt>Website</dt><dd><a href="{url}">{url_text}</a></dd>
</dl>
<h2>Facts</h2>
{facts}"#
    );
    page(&e.name, "/", &body)
}

fn llms(c: &Catalog) -> String {
    let mut s = format!(
        "# Project Mercury\n\n> A sourced catalog of homeschool resources and of what each US state asks of home educating families. Every fact links to its source.\n> Full data: {BASE_URL}/dump.json\n\n## Resources\n\n"
    );
    for e in &c.entries {
        let summary = e.summary.split_whitespace().collect::<Vec<_>>().join(" ");
        s += &format!(
            "- [{}]({BASE_URL}/entries/{}/): {summary}\n",
            e.name,
            slug(&e.id)
        );
    }
    if !c.states.is_empty() {
        s += "\n## States\n\nInformational, not legal advice.\n\n";
        for st in &c.states {
            let summary = st.summary.split_whitespace().collect::<Vec<_>>().join(" ");
            let (code, level) = (st.code.to_ascii_lowercase(), st.regulation().label());
            s += &format!(
                "- [{}]({BASE_URL}/states/{code}/): {level}. {summary}\n",
                st.name
            );
        }
    }
    s
}
