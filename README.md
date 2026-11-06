# Project Mercury

A navigable, sourced catalog of favorite homeschool resources, plus what each US
state and DC asks of home educating families, written in Rust. Every factual claim
cites a source and says how it was obtained.

## Layout

- `data/entries/*.yaml`: one catalog entry per file
- `data/states/<xx>.yaml`: one record per state or DC (`xx` is the lowercase USPS code)
- `data/sources/*.yaml`: a list of cited sources per file (usually one file per publisher)
- `crates/core` (`project-mercury-core`): types, YAML loading, `validate()`, and a pure `search()`
- `crates/cli` (`project-mercury-cli`, binary `project-mercury`): command-line interface
- `crates/web` (`mercury-site`): static website generator

## Provenance

Each claim has a `provenance`:

- `measured`: seen on the cited page on `checked_on` (required for measured claims)
- `inferred`: derived from what the source says, not stated verbatim
- `guess`: not verified

`validate` checks that every claim cites an existing source, that every measured
claim has `checked_on`, and that entry and source ids are unique.

## Usage

```sh
cargo run -p project-mercury-cli -- search --age 9 --subject math
cargo run -p project-mercury-cli -- search phonics --json
cargo run -p project-mercury-cli -- show math-academy
cargo run -p project-mercury-cli -- validate
```

`search [QUERY] [--subject S] [--age N] [--approach A] [--skill-level L]
[--format F] [--cost-tier T] [--json]` matches QUERY case-insensitively against
name, summary and subjects. `--data DIR` overrides the default `./data`.

## Facets

Optional at-a-glance fields: `best_at`, `most_useful_when`, `skill_levels`
(beginner/intermediate/advanced), `format` (online/in_person/hybrid), `cost_tier`
(free, low <$50/mo, mid $50-300/mo, high >$300/mo), `parent_involvement`,
`screen_time` (low/medium/high; screen time low is up to ~30 min/day, high over 2 h)
and `pace` (self_paced/scheduled). `validate` requires a claim with `field` equal to
the facet name for every facet that is set.

## States

Each state record has `compulsory_ages`, `funding` and `public_school_access`
(`{ available, detail }`), and one or more `pathways`: the legal routes to home
educating (home school statute, umbrella or church school, tutor, an ESA that
satisfies attendance on its own). Each pathway lists seven duties as
`{ required, detail }`: `notice`, `approval`, `assessment`, `subjects`,
`instruction_time`, `teacher_qualifications` and `records`.

A duty that applies only in some cases (certain grades, some families) is
`required: true` with the conditions in `detail`. The exception is notice: a
filing owed only on withdrawal from public school, on request, or to join public
school activities is `required: false`, with the condition in `detail`.

`validate` requires a claim for `compulsory_ages`, `funding`,
`public_school_access` and every `pathways.<id>.<duty>`. State claims must quote
the cited page verbatim (`quote`), carry `checked_on` as `YYYY-MM-DD`, be
`measured` or `inferred` (not `guess`), and cite a source of type `statute` or
`agency`. Advocacy summaries are never cited.

Regulation is computed, not stored. A pathway is **high** if the program must be
approved, otherwise **moderate** if it requires assessment, otherwise **low** if
it requires notice or a teacher qualification, otherwise **no notice**. A state
takes its least regulated pathway's level. Optional funding programs' strings
(tests, reports) go in `funding.detail`; they do not change a pathway's duties.

State pages are informational, not legal advice.

## Website

`mercury-site` renders the catalog to a static site in `dist/` (gitignored): a home
page of resource cards (best at, most useful when, age bar, involvement and screen-time
meters, cost tier, skill levels) with an age-coverage chart and a faceted filter bar
(age, subject and each facet, with counts; a small inline script filters live, and
without JS every card shows), one page per entry with a sourced facts table, one page
per approach, a states page (tile map by regulation level and a summary table) with
one page per state (pathways and duties with inline source links, quotes in a
collapsible table), plus `styles.css`, `404.html`, `llms.txt` and `dump.json` (the
whole catalog as JSON). Light and dark follow `prefers-color-scheme`.
The catalog is validated first; the build fails on any validation error.

```sh
cargo run -p mercury-site -- --data data --out dist
python3 -m http.server --directory dist 8000   # local preview
```

### Deploy

`wrangler.jsonc` configures an assets-only Cloudflare Worker named
`project-mercury` that serves `./dist` on `projectmercury.mistystep.io`. After
building:

```sh
npx wrangler deploy
```

Wrangler uses your own Cloudflare login (`npx wrangler login`) or a
`CLOUDFLARE_API_TOKEN`; no account id or secrets live in the repo.

## Licence

MIT, for both code and data. See [LICENSE-MIT](LICENSE-MIT).
