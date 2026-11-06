# CLI search, show, and validate

A terminal user searches the catalog, reads a resource's sourced claims, exports JSON, and checks whether catalog data is internally consistent.

## Sub-features

- `cli-search-text` searches case-insensitively by text and returns an empty set for no matches.
- `cli-search-facets` intersects age, subject, approach, skill, format, and cost filters.
- `cli-show` displays one resource with facts and sources in text or JSON.
- `cli-validate` accepts the committed catalog.

## How to get to it (user POV)

- Run `project-mercury search`, `project-mercury show ID`, or `project-mercury validate` in a terminal.
- Use the global `--data DIR` flag for a different catalog directory.
- The README's `cargo run -p project-mercury-cli -- ...` commands reach the same executable. Verification builds once and drives `target/debug/project-mercury`.

## Driving it with shell

Preconditions:

- Launch built the CLI in the same worker as the website.
- Run commands from `WORKER_ROOT`. Prefix every command below with `.agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" cli --name UNIQUE_NAME --` through `boat exec` as shown in the skill.
- Read command transcripts after execution. A helper exit of zero does not replace inspection of the recorded result.

- **Text search.** Run `search PHONICS --json`. Require exit `0`, a JSON array, and entries matching phonics in the case-insensitive name, summary, or subjects. Compare with `search phonics --json`. Run `search verification-no-such-resource --json` and require exit `0` with `[]`.
- **Age and subject.** Run `search --age 9 --subject math --json`. Require Math Academy and compare the complete ID set with matching served-catalog entries. Every result includes math and age 9. No nonmatching result may remain.
- **Facets.** Run `search --age 9 --subject math --approach mastery --skill-level advanced --format online --cost-tier low --json`. Require Math Academy and compare the full matching set with the catalog. These flags intersect rather than replace one another.
- **Text show.** Run `show math-academy`. Require exit `0`, Math Academy's name and URL, subjects, approaches, cost, and claim lines with provenance and source URLs. The current inferred claims without dates display `unchecked`.
- **JSON show.** Run `show math-academy --json`. Require exit `0` and one object with ID `math-academy`, the committed ages, facets, and claims. JSON contains source IDs. Text show resolves them to URLs.
- **Validate.** Run `validate`. Require exit `0` and `ok: ... entries, 51 states, ... sources`, with counts matching the served catalog. Run the separate [unsourced claim rejection](unsourced-claim.md) for the negative path and recovery.
- **Explicit data directory.** Run `--data data show math-academy --json`. Require the same object as the default-directory command.

## Gotchas

- CLI search and show load data but do not automatically run `validate`. Successful search does not certify provenance.
- No-match search is an empty success, not a validation failure.
- The CLI has no states subcommand. State journeys are browser journeys.
- `--json` is per search or show. `--data` is global.
- Do not assert fixed source or resource counts after catalog additions. Compare with the actual committed catalog.
