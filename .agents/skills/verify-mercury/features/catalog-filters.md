# Resource catalog filters

A reader narrows the home-page resource cards by age and facets, sees matching counts and age coverage, and clears the selection without leaving the page.

## Sub-features

- `catalog-baseline` shows every committed resource with no active filter.
- `catalog-age-subject` intersects age and subject selections.
- `catalog-facets` combines different facet axes with AND and values within one axis with OR.
- `catalog-availability` updates chip counts, disables impossible unselected choices, and preserves selected choices.
- `catalog-reset` clears the controls, restores all cards and coverage rows, and hides Clear.

## How to get to it (user POV)

- Open the website root `/`.
- Choose Resources in the main navigation from an entry, state, or approach page.
- Choose the Project Mercury wordmark to return home.

`/entries/` is an unfiltered resource listing. It is not the interactive home page.

## Driving it with OMP browser

Preconditions:

- Doctor passes and the forwarded run identity matches the launch.
- The committed catalog is available through `APP_URL/dump.json` for read-only expected-result calculation.
- Start the Evidence video and action log before selecting a control.

- **Baseline.** Run `await tab.goto(appUrl + '/')`. Require the Resources heading, visible `form[aria-label="Filters"]`, `#age` value `''`, and all `.chip` buttons with `aria-pressed="false"`. Read the catalog through the user-facing `dump.json`. Require `#count` and `#grid > li:not([hidden])` to equal its entry count. Require `#clear` and `#empty` hidden.
- **Age.** Run `await tab.select('#age', '9')`. Wait for the displayed count to equal the number of entries with `ages.min <= 9 && ages.max >= 9`. Compare the visible card IDs to those exact entry IDs prefixed with `r-`. Age-coverage cells for age 9 have `.now`.
- **Subject.** Run `await tab.click('.chip[data-axis="subject"][data-value="math"]')`. Require `aria-pressed="true"`. Compare visible IDs and `#count` with catalog entries whose subject includes math and whose age range includes 9. Math Academy must remain. Require each `#cover li[data-for]` to have `.off` exactly when its card is hidden.
- **Other facets.** Choose an enabled `.chip[data-axis="format"][data-value="online"]`. Compare visible cards with the same expected set restricted to `format === 'online'`. Read each chip's `.n` and `disabled` state. A chip counts results with the other axes applied, ignoring its own current axis. An unselected zero-count chip is disabled. A selected chip stays enabled so it can be cleared. Repeat for skill, cost, parent involvement, and screen time when those axes are affected by the change.
- **OR within an axis.** From baseline, choose two enabled subject chips. Require the union of their subjects, not their intersection. Then choose an enabled facet on another axis and require its intersection with that union.
- **Reset.** Run `await tab.click('#clear')`. Require `#age` value `''`, no pressed chip, all cards visible, no coverage row `.off`, no `.now` age cell, `#count` equal to the original catalog size, and Clear hidden. Save the resulting snapshot and screenshot.

## Gotchas

- Browser selectors must target `#grid > li:not([hidden])`. Counting every card ignores the filter result.
- Chip accessible names contain live counts, so exact button-name strings are unstable. The axis and value attributes are stable.
- Options with no possible results are disabled. Do not force-click one or edit the DOM to manufacture a No matches state.
- Without JavaScript, every card is shown and the filters are hidden. That fallback does not prove interactive filtering.
- The current seed does not guarantee a reachable empty result through enabled controls. If a changed journey requires No matches and no valid control sequence reaches it, report that missing precondition. The separate unsourced-claim journey supplies a deterministic rejection path.
