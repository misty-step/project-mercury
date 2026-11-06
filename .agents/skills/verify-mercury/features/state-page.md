# State page

A reader compares a state's legal pathways, reads each duty and its source, and expands the full source-and-quote table.

## Sub-features

- `state-pathways` shows California's private-school and tutor pathways.
- `state-duties` shows seven required or not-required duties per pathway.
- `state-citations` exposes source links and quotes beside the relevant claims.
- `state-sources` expands and collapses the complete sourced facts table.
- `state-return` returns to the states comparison page.

## How to get to it (user POV)

- Choose California in the `/states/` tile map.
- Choose California in the `/states/` summary table.
- Open `/states/ca/` directly.

## Driving it with OMP browser

Preconditions:

- Doctor passes. California retains `private-school` and `credentialed-tutor` pathways in the committed data.
- Read California and its referenced sources from the served `dump.json` for expected details and URLs.

- **Open.** Follow either mapped California link and require the California heading, CA code, regulation badge, compulsory ages, and informational disclaimer.
- **Pathways.** Require `#p-private-school` with heading Private full-time day school and `#p-credentialed-tutor` with heading Instruction by tutor. The least-regulated-path explanation links to `#p-private-school`. Click that explanation link and require its fragment and target section.
- **Duties.** In each `.path`, require seven `.duty` rows labeled Notice, Approval, Assessment, Subjects, Instruction time, Teacher qualifications, and Records. Compare each row's required status and detail with the catalog. The tutor's instruction time is required. The private school's instruction time has no numerical minimum. Check text and accessible status, not only checkmark color.
- **Inline sources.** Read `.cite` links under each pathway, the compulsory-age display, and Also consider. Require the expected source URLs, nonempty `Source: ...` accessible labels, and quote text in their `title` attributes. Open an affected external source with a separate read-only request if reachability is in scope. Record external failures distinctly.
- **Expand.** Require `details.all` initially closed. Run `await tab.click('details.all summary')`. Wait for `details.all[open]` and a visible `details.all table.facts`. Require its row count to equal California's claims. Rows show claim value, quoted source text in `q.quote`, provenance, publisher link, and checked date. Compare the rows to the catalog, including source IDs resolved to URLs.
- **Collapse.** Click `details.all summary` again. Require no `open` attribute and a hidden facts table. Capture expanded and collapsed states with the action and assertion log.
- **Return.** Run `await tab.click('a.crumb[href="/states/"]')`. Require the states index with its map and table.

## Gotchas

- The full facts table is inside a native details element. It exists in the DOM while collapsed, so element count alone does not prove expansion.
- California has multiple pathways. Verifying only the first one misses tutor-specific duties.
- An inline citation's quote is a tooltip. The expanded table is the legible quote evidence.
- Source text is informational evidence, not a legal-review guarantee. No human or legal review is part of this journey.
- The source table's summary includes a live claim count. Select `details.all summary` rather than an exact accessible name containing that count.
