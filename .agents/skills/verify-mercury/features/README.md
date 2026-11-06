# Mercury verification map

This map is the maintained source for Mercury's main user journeys. Launch the app with [the skill](../SKILL.md), pass Doctor, and read the matching recipe before driving it.

## Baseline preconditions

- Serve the committed catalog from an owned worker through `APP_URL`, normally `http://127.0.0.1:8765`.
- Use the built `target/debug/project-mercury` and `target/debug/mercury-site` binaries from that same worker checkout.
- Require the served run identity to match `launch.json`. Evidence stays outside the checkout.
- The seed catalog includes Math Academy and California. California has private-school and credentialed-tutor pathways.
- No account, credentials, mutable service, or family-specific data is needed.

## Driving conventions

- Start browser recipes at the named entry point. Read the current page before interacting.
- Use OMP browser controls and stable selectors. Read-only DOM assertions are allowed. Do not alter the page to make an assertion pass.
- Use the helper's `cli --name NAME -- ...` command for CLI transcripts. Every artifact name must be unique within the run.
- Capture the action and result, not only a final screenshot. Follow the skill's Evidence and Cleanup sections.
- An unavailable source website is an external-link gap. A rendered source link does not prove the source is reachable or legally correct.
- Record each attempted entry point separately. A direct route does not prove its navigation link.

## Proof and failure reporting

For each driven sub-feature, record its ID, entry point, source revision, expected result, observed result, and artifact. Capture browser interactions with a video, action log, ARIA snapshot, and before and after screenshots. CLI proof includes the invocation, output, and exit status. Rejection proof also includes the mutated disposable fixture, absence of generated output, restored validation, and unchanged original fixture.

If committed data lacks a required precondition, report the journey as failed and record the reason in that feature's Gotchas. Do not substitute another journey or weaken an assertion. Unexecuted entry points stay unverified.

## Features

- [Resource catalog filters](catalog-filters.md) covers home-page age and facet selection, counts, coverage, disabled chips, and reset.
- [Entry page facts](entry-facts.md) covers resource navigation, sourced claims, provenance, dates, website links, and approach entry points.
- [States map and table](states-index.md) covers all 50 states and DC, regulation levels, and both navigation routes.
- [State page](state-page.md) covers California's pathways, duties, inline citations, and collapsible source quotes.
- [CLI search, show, and validate](cli.md) covers text and facet search, resource facts, JSON output, and a valid catalog.
- [Unsourced claim rejection](unsourced-claim.md) covers validation and generation refusing a broken source reference, then recovery.
