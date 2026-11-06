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

For running-app proof, use [verify-mercury](.agents/skills/verify-mercury/SKILL.md) to launch the site and CLI, drive mapped journeys, and retain evidence.

### HTML safety

Run the bounded HTML escaping and external-link properties with:

```sh
cargo test --locked -p mercury-site --test html_safety
```

The [HTML safety skill](.agents/skills/verify-html-safety/SKILL.md) records the
covered renderer functions, input bounds, tool versions, and scratch defects the
check must reject. These properties supplement the real-site checks.

### Deploy

`wrangler.jsonc` configures an assets-only Cloudflare Worker named
`project-mercury` that serves `./dist` on `projectmercury.mistystep.io`. After
building:

```sh
npx wrangler deploy
```

Wrangler uses your own Cloudflare login (`npx wrangler login`) or a
`CLOUDFLARE_API_TOKEN`; no account id or secrets live in the repo.

### Release identity

Every build writes `/version.json` with `version`, `commit`, `environment`,
`built_at` as Unix seconds, and `release`. Every HTML page includes matching
`mercury-version`, `mercury-commit`, `mercury-release`, and `mercury-environment`
meta tags. The Sentry release name is `project-mercury@<version>+<commit>`.

The generator reads these build-time variables:

- `MERCURY_VERSION` defaults to the Cargo package version.
- `MERCURY_COMMIT` takes precedence over `GITHUB_SHA`, then `git rev-parse HEAD`.
  Without any of those, the build prints a warning and records `unknown`.
- `MERCURY_ENVIRONMENT` defaults to `development`. Set it to `production` for
  deployment, or to `verification` for telemetry checks that must not page the owner.

```sh
MERCURY_ENVIRONMENT=production cargo run -p mercury-site -- --data data --out dist
```

## Telemetry

Sentry receives browser errors in the dedicated
[`misty-step/project-mercury` project](https://misty-step.sentry.io/settings/projects/project-mercury/).
Its async, errors-only loader does not gate the page or filters. The client sets
`sendDefaultPii: false`, disables breadcrumbs, sessions, tracing, and replay, and
sends only error type, up to 20 stack locations, page URL, release, environment,
and the `project: project-mercury` tag. It omits error messages, function names,
user information, request headers, form data, and arbitrary error context.
URLs omit query strings and fragments. The project also scrubs IP addresses.
Production samples errors at 25%, with at most five events per page load.
Other environments keep all errors within the same five-event limit.

Sentry uptime monitors `10871013` and `10871014` check `/` and `/states/` every
60 seconds with a 10-second timeout. Three failed checks open an incident and one
successful check recovers it. They require HTTP 200, use the `production`
environment, and do not capture response bodies or tracing data. Alert rule
`6300387` covers browser
errors and both monitors. It copies the fleet's owned response actions,
the Conductor webhook and owner email, with a 30-minute repeat limit.
Provider-side retention and account-wide quotas remain the Sentry organization's
settings. Its current plan does not expose a per-key error-rate limit.

PostHog is optional. Set the public `POSTHOG_KEY` at build time to use the shared
Misty Step PostHog project. `POSTHOG_HOST` defaults to `https://us.i.posthog.com`.
Without a key, the generator emits no PostHog script or HTML script tag.
Do not commit the key. Every event has `project: project-mercury`, `release`,
and `environment` super-properties.

The analytics events and their event-specific properties are:

| Event | Properties |
| --- | --- |
| `$pageview` | None |
| `filter_changed` | `axis`, `value` |
| `state_viewed` | `code`, `regulation` |
| `source_clicked` | `page_type`, `source_type`, `host` |
| `state_map_tile_clicked` | `code` |

`filter_changed` uses an empty `value` when a selection is removed. Clear uses
`axis: all` and an empty `value`. Values come from the catalog's filter choices,
not free text. Source events include only the source host, not its path or URL.
The client allows at most 50 analytics events per page load.

PostHog uses memory-only persistence and no person profiles, autocapture,
session recording, performance capture, referrers, or campaign parameters.
The outgoing event property allowlist retains only those listed above, a
query-free and fragment-free current URL and path, and the SDK's public token,
version, and page-local anonymous identifier. It requests `ip: false` and sets
`$geoip_disable: true`. Current PostHog versions require **Discard client IP
data** in the shared project's settings to discard the connection IP.
That setting and provider retention need verification by the project owner.
Analytics never records a family, name, email, form content, or search query.
It does not create cookies or cross-session identifiers.

The pinned PostHog SDK retries failed HTTP requests at most 10 times, or three
times when no HTTP response arrives, with backoff. Queues live only in memory.
Sentry drops failed sends rather than maintaining a persistent retry queue.

Both telemetry clients load asynchronously. Blocked SDKs or ingestion requests
do not stop navigation, filtering, or the no-JavaScript catalog.

## Licence

MIT, for both code and data. See [LICENSE-MIT](LICENSE-MIT).
