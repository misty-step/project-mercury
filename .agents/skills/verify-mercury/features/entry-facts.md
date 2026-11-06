# Entry page facts

A reader opens a resource, inspects its useful-at-a-glance details, and traces factual claims to publishers with provenance and checked dates.

## Sub-features

- `entry-open` opens Math Academy from resource cards and its direct URL.
- `entry-details` shows its summary, ages, subjects, approaches, cost, website, and facets.
- `entry-facts` shows every claim with value, provenance, source, and checked date.
- `entry-approach` follows an approach to its listing and opens a resource again.

## How to get to it (user POV)

- Choose Math Academy on the home-page card or `/entries/` resource listing.
- Open `/entries/math-academy/` directly.
- Choose Approaches in the main navigation, choose mastery, then Math Academy.
- From Math Academy, choose its mastery approach link.

## Driving it with OMP browser

Preconditions:

- Doctor passes and the committed Math Academy entry exists.
- Read its record and referenced sources from the served `dump.json` for expected values.

- **Open.** From `/`, run `await tab.click('#r-math-academy h3 a')`. Wait for `/entries/math-academy/` and require `h1` text `Math Academy`. Repeat the `/entries/` and direct-route entry points when navigation changes.
- **Details.** Read `.hero`, `.summary`, and `dl.meta`. Require the committed cost text, subjects, and approach names. Require the Website link to have `https://www.mathacademy.com/` as its href. The entry's age bar shows its committed minimum and maximum ages.
- **Facts.** Read `table.facts thead` and require Field, Value, Provenance, Source, and Checked. Require `table.facts tbody tr` count to equal the entry's claims. Compare each row with its claim and referenced publisher. Measured claims have their checked date. Inferred claims without a checked date display a dash. Source links use the corresponding source URL, not an empty link or `#`.
- **Approach link.** Run `await tab.click('dl.meta a[href="/approaches/mastery/"]')`. Require the mastery heading and Math Academy card. Open that card and require the entry heading again.
- **Approach index.** Open `/approaches/`, choose `a[href="/approaches/mastery/"]`, and repeat the card navigation. Record this separately from the entry-page approach link.
- **Proof.** Save the entry's ARIA snapshot, facts-table screenshot, and action log. Open one source URL through a separate read-only request or dedicated tab if source reachability is part of the changed criterion. Do not navigate the owned local-app tab away from its allowed host.

## Gotchas

- A resource's website and its individual claim sources are different links.
- Provenance describes how a claim was obtained. A rendered badge does not independently verify the claim's truth.
- An inferred claim may legitimately omit a checked date. Do not require every date cell to be populated.
- Approach listings have cards but no live filter controls. Return to `/` to verify filters.
- External source availability is outside the static generator. Report an unreachable source separately rather than hiding it.
