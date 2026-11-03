# Project Mercury

A navigable, sourced catalog of favorite homeschool resources, written in Rust.
Every factual claim in the catalog cites a source and says how it was obtained.

## Layout

- `data/entries/*.yaml`: one catalog entry per file
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

## Website

`mercury-site` renders the catalog to a static site in `dist/` (gitignored): a home
page with an age/subject filter (works without JS; a small inline script narrows the
list), one page per entry with a sourced facts table, one page per approach, plus
`styles.css`, `404.html`, `llms.txt` and `dump.json` (the whole catalog as JSON).
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
