# States map and table

A reader compares homeschool regulation across the 50 states and DC, then opens a state from either its tile or summary-table row.

## Sub-features

- `states-coverage` shows 51 tiles and 51 table rows with no missing or duplicate jurisdiction.
- `states-summary` shows regulation, least regulated path, compulsory ages, funding, and public-school access.
- `states-map-open` navigates through a tile.
- `states-table-open` navigates through a named table link.

## How to get to it (user POV)

- Choose States in `nav[aria-label="Main"]`.
- Open `/states/` directly.
- Choose the States breadcrumb from any state page.

## Driving it with OMP browser

Preconditions:

- Doctor passes and the served catalog includes all 51 state records.

- **Open.** From `/`, run `await tab.click('nav[aria-label="Main"] a[href="/states/"]')`. Require the States heading and visible informational disclaimer.
- **Coverage.** Require `ol[aria-label="States by regulation"] a.tile` and `table.sl tbody tr` each to have 51 elements. Compare the href sets in both representations with `/states/` plus each lowercase catalog code plus `/`. Require all 51 unique routes and no placeholder tile.
- **Summary.** Read `table.sl thead`. Require State, Regulation, Least regulated path, Compulsory ages, State funding, and Public classes or sports. Compare California's row with its committed record. Its regulation is set by the least regulated pathway, not an average of the pathways. Read badge `title` attributes for the rule explanation. Require the legend counts to sum to 51.
- **Map route.** Run `await tab.click('ol.tiles a[href="/states/ca/"]')`. Require `/states/ca/` and `h1` text California. Use the `a.crumb[href="/states/"]` breadcrumb to return.
- **Table route.** Run `await tab.click('table.sl a[href="/states/ca/"]')`. Require the same route and heading. This is a separate navigation proof.
- **Proof.** Capture the index snapshot, map and table screenshots, and both click outcomes in the action log and video. Verify direct entry and the main-navigation entry separately when either changes.

## Gotchas

- Tile text abbreviates the state code. Its accessible name also includes the full name and regulation level.
- DC is a first-class jurisdiction. A check for only 50 records misses it.
- The map is a tile grid, not a geographically exact map.
- Funding requirements do not determine the regulation badge. Use the pathway duties described in the README.
- These pages are informational, not legal advice. This journey checks rendered data and navigation, not legal correctness.
