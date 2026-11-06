//! State pages: a tile map and table of every state, and one page per state.

use project_mercury_core::{Availability, Catalog, Pathway, Regulation, State};

use super::{esc, facts, href, icon, marks, page, AGES};

/// Tile-grid map positions: (code, column, row), 11 columns by 8 rows.
#[rustfmt::skip]
const TILES: [(&str, u8, u8); 51] = [
    ("AK", 0, 0), ("ME", 10, 0),
    ("VT", 9, 1), ("NH", 10, 1),
    ("WA", 0, 2), ("ID", 1, 2), ("MT", 2, 2), ("ND", 3, 2), ("MN", 4, 2), ("IL", 5, 2),
    ("WI", 6, 2), ("MI", 7, 2), ("NY", 8, 2), ("RI", 9, 2), ("MA", 10, 2),
    ("OR", 0, 3), ("NV", 1, 3), ("WY", 2, 3), ("SD", 3, 3), ("IA", 4, 3), ("IN", 5, 3),
    ("OH", 6, 3), ("PA", 7, 3), ("NJ", 8, 3), ("CT", 9, 3),
    ("CA", 0, 4), ("UT", 1, 4), ("CO", 2, 4), ("NE", 3, 4), ("MO", 4, 4), ("KY", 5, 4),
    ("WV", 6, 4), ("VA", 7, 4), ("MD", 8, 4), ("DE", 9, 4),
    ("AZ", 1, 5), ("NM", 2, 5), ("KS", 3, 5), ("AR", 4, 5), ("TN", 5, 5), ("NC", 6, 5),
    ("SC", 7, 5), ("DC", 8, 5),
    ("OK", 3, 6), ("LA", 4, 6), ("MS", 5, 6), ("AL", 6, 6), ("GA", 7, 6),
    ("HI", 0, 7), ("TX", 3, 7), ("FL", 8, 7),
];

const DUTY_LABELS: [&str; 7] = [
    "Notice",
    "Approval",
    "Assessment",
    "Subjects",
    "Instruction time",
    "Teacher qualifications",
    "Records",
];

const NOTICE: &str = r#"<p class="note">Informational, not legal advice. Laws change; read the linked statute or ask the state agency before relying on this.</p>"#;

fn path(s: &State) -> String {
    format!("/states/{}/", s.code.to_ascii_lowercase())
}

/// Level badge; the tooltip states the rule that sets the level.
pub(crate) fn badge(r: Regulation) -> String {
    let (k, label, rule) = (r.as_str(), r.label(), r.rule());
    format!(r#"<span class="lv lv-{k}" title="{label}: {rule}">{label}</span>"#)
}

/// Every output file for the states section.
pub(crate) fn pages(c: &Catalog) -> Vec<(String, String)> {
    let mut states: Vec<&State> = c.states.iter().collect();
    states.sort_by(|a, b| a.name.cmp(&b.name));
    let mut files = vec![("states/index.html".to_string(), index(&states))];
    for s in states {
        let rel = format!("states/{}/index.html", s.code.to_ascii_lowercase());
        files.push((rel, state_page(c, s)));
    }
    files
}

fn tile_map(states: &[&State]) -> String {
    let mut tiles = String::new();
    for (code, col, row) in TILES {
        let (col, row) = (col + 1, row + 1);
        let pos = format!("grid-column:{col};grid-row:{row}");
        tiles += &match states.iter().find(|s| s.code == code) {
            Some(s) => {
                let (r, name, href) = (s.regulation(), esc(&s.name), path(s));
                let (k, label) = (r.as_str(), r.label());
                format!(
                    r#"<li style="{pos}"><a class="tile lv-{k}" href="{href}" title="{name}: {label}"><span aria-hidden="true">{code}</span><span class="sr">{name}, {label}</span></a></li>"#
                )
            }
            None => format!(
                r#"<li style="{pos}"><span class="tile" title="{code}: not yet researched">{code}</span></li>"#
            ),
        };
    }
    let mut legend = String::new();
    for r in Regulation::ALL {
        let n = states.iter().filter(|s| s.regulation() == r).count();
        let (k, label, rule) = (r.as_str(), r.label(), r.rule());
        legend += &format!(
            r#"<li><i class="sw lv-{k}"></i><span>{label}</span><span class="n">{n}</span><span class="rule">{rule}</span></li>"#
        );
    }
    format!(
        r#"<div class="smap"><ol class="tiles" aria-label="States by regulation">{tiles}</ol><ul class="legend">{legend}</ul></div>"#
    )
}

fn yes_no(a: &Availability) -> &'static str {
    ["No", "Yes"][a.available as usize]
}

fn index(states: &[&State]) -> String {
    let n = states.len();
    let mut rows = String::new();
    for s in states {
        let (name, href, r) = (esc(&s.name), path(s), badge(s.regulation()));
        let easiest = s.easiest().map(|p| esc(&p.name)).unwrap_or_default();
        let (a, b) = (s.compulsory_ages.min, s.compulsory_ages.max);
        let (f, p) = (yes_no(&s.funding), yes_no(&s.public_school_access));
        rows += &format!(
            r#"
<tr><td><a href="{href}">{name}</a></td><td>{r}</td><td>{easiest}</td><td class="mono">{a}–{b}</td><td>{f}</td><td>{p}</td></tr>"#
        );
    }
    let body = format!(
        r#"<div class="head"><h1>States</h1><span class="count">{n}</span></div>
{NOTICE}
{map}
<table class="facts sl">
<thead><tr><th>State</th><th>Regulation</th><th>Least regulated path</th><th>Compulsory ages</th><th>State funding</th><th>Public classes or sports</th></tr></thead>
<tbody>{rows}
</tbody>
</table>"#,
        map = tile_map(states)
    );
    page("States", "/states/", &body)
}

/// Links to every source backing `field`; each tooltip is the quote.
fn cites(c: &Catalog, s: &State, field: &str) -> String {
    let mut out = String::new();
    for cl in s.claims.iter().filter(|cl| cl.field == field) {
        let Some(src) = c.source(&cl.source_id) else {
            continue;
        };
        let quote = esc(cl.quote.as_deref().unwrap_or(&cl.value));
        let (url, publisher, ext) = (href(&src.url), esc(&src.publisher), icon("external"));
        out += &format!(
            r#"<a class="cite" href="{url}" title="“{quote}” ({publisher})" aria-label="Source: {publisher}">{ext}</a>"#
        );
    }
    out
}

/// One duty or consideration row: a check or dash, the label, then the detail and its sources.
fn row(on: bool, label: &str, state: &str, detail: &str, cites: &str) -> String {
    let (cls, ic) = if on {
        ("on", icon("check"))
    } else {
        ("", icon("minus"))
    };
    let detail = esc(detail);
    format!(
        r#"<div class="duty {cls}"><dt>{ic}{label}<span class="sr">: {state}</span></dt><dd>{detail}{cites}</dd></div>"#
    )
}

fn pathway(c: &Catalog, s: &State, p: &Pathway) -> String {
    let (id, name, summary, r) = (
        esc(&p.id),
        esc(&p.name),
        esc(&p.summary),
        badge(p.regulation()),
    );
    let mut duties = String::new();
    for ((key, d), label) in p.duties().into_iter().zip(DUTY_LABELS) {
        let state = ["Not required", "Required"][d.required as usize];
        let cites = cites(c, s, &format!("pathways.{}.{key}", p.id));
        duties += &row(d.required, label, state, &d.detail, &cites);
    }
    format!(
        r#"
<section class="path" id="p-{id}" aria-labelledby="h-{id}">
<div class="row"><h3 id="h-{id}">{name}</h3>{r}</div>
<p>{summary}</p>
<dl class="duties">{duties}</dl>
</section>"#
    )
}

/// Display label for a claim field, using the pathway's name.
fn field_label(s: &State, field: &str) -> String {
    match field {
        "compulsory_ages" => "Compulsory ages".into(),
        "funding" => "State funding".into(),
        "public_school_access" => "Public classes or sports".into(),
        f => {
            let mut parts = f.splitn(3, '.').skip(1);
            let (id, duty) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            let name = s
                .pathways
                .iter()
                .find(|p| p.id == id)
                .map_or(id, |p| &p.name);
            let i = Pathway::DUTIES.iter().position(|d| *d == duty);
            format!("{name}: {}", i.map_or(duty, |i| DUTY_LABELS[i]))
        }
    }
}

fn state_page(c: &Catalog, s: &State) -> String {
    let (name, code, summary) = (esc(&s.name), esc(&s.code), esc(&s.summary));
    let r = s.regulation();
    let level = badge(r);
    let easiest = match (s.easiest(), s.pathways.len()) {
        (Some(p), 2..) => format!(
            r##"<p class="why">Set by the least regulated path, <a href="#p-{}">{}</a>: {}.</p>"##,
            esc(&p.id),
            esc(&p.name),
            r.rule()
        ),
        _ => format!(r#"<p class="why">{}.</p>"#, capitalize(r.rule())),
    };
    let (a, b) = (s.compulsory_ages.min, s.compulsory_ages.max);
    let cells = marks(AGES.map(|y| (a..=b).contains(&y)));
    let age_cites = cites(c, s, "compulsory_ages");
    let ages = format!(
        r#"<div class="age" title="Compulsory school ages {a}–{b}"><span class="cells" aria-hidden="true">{cells}</span><span class="al"><span class="sr">Compulsory ages </span>{a}–{b}</span></div><span class="cites">{age_cites}</span>"#
    );
    let paths: String = s.pathways.iter().map(|p| pathway(c, s, p)).collect();
    let paths_head = ["Path", "Ways to home educate"][(s.pathways.len() > 1) as usize];
    let avail = |a: &Availability, label: &str, field: &str| {
        row(
            a.available,
            label,
            yes_no(a),
            &a.detail,
            &cites(c, s, field),
        )
    };
    let more = avail(&s.funding, "State funding", "funding")
        + &avail(
            &s.public_school_access,
            "Public classes or sports",
            "public_school_access",
        );
    let n = s.claims.len();
    let facts = facts(c, &s.claims, |f| field_label(s, f));
    let body = format!(
        r#"<a class="crumb" href="/states/">States</a>
<div class="hero">
<div class="row">{level}<span class="kind">{code}</span></div>
<h1>{name}</h1>
<p class="lede">{summary}</p>
{easiest}
<div class="glance">{ages}</div>
</div>
<h2>{paths_head}</h2>
{paths}
<h2>Also consider</h2>
<dl class="duties wide">{more}</dl>
<details class="all"><summary>Sources and quotes <span class="n">{n}</span></summary>
{facts}
</details>
{NOTICE}"#
    );
    page(&s.name, "/states/", &body)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}
