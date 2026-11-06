---
name: verify-html-safety
description: Check Mercury's HTML escaping and external links when changing the Rust renderer, catalog string fields, or the HTML safety properties.
---

# Verify HTML safety

1. Read the coverage and bounds below. Compare the covered functions with the sync commit before trusting the check.
2. From the repository root, run the command below with the repository's Rust toolchain. Use the slice's Boat worker when workstation policy requires isolation.
3. Require all six tests to pass. On failure, retain the test name, slot or URL, counterexample, source commit, and tool versions outside Git.
4. Reproduce a counterexample through `mercury_site::render`. A data string that adds an element, attribute, or executable text is a finding. An external data link that escapes the allowlist is a finding. Do not weaken a property to pass.
5. If a change intentionally adds markup, inspect the new interpolation contexts. Update `TEXT_SLOTS`, the fixture, and the coverage record together. A fixture reference that no longer reaches its renderer is a modeling error, not a product finding.
6. After a covered code change, exercise the scratch breaks below and restore the source. Move the sync commit forward only after confirming that the properties still describe that commit.

```sh
cargo test --locked -p mercury-site --test html_safety
```

## Properties and harm

- P1. `esc` preserves text in HTML element content and single-quoted and double-quoted attributes without leaving raw HTML delimiters. A broken encoder can let a catalog edit execute code in a visitor's browser.
- P2. Changing a covered catalog string cannot change the parsed element tree, attribute names, or existing script and style text. A missing escape at one call site can bypass a correct encoder.
- P3. Every data-derived external anchor has an HTTP or HTTPS URL accepted by `href`, or exactly `#`, and URL data cannot change document structure. An unsafe scheme or quote breakout can execute code when a visitor follows a source.

Internal navigation intentionally uses root-relative paths and pathway fragments. The HTTP-or-`#` contract applies to external data links, not those internal links.

## Coverage record

Last confirmed source commit is `25a2313f76baad7d8d8968377dfe5155223ddb6f`. This commit identifies the covered production code, which this change does not modify.

| Property | Production code | Primary check |
| --- | --- | --- |
| P1 | `crates/web/src/lib.rs`, `esc` | `escaped_delimiter_language_is_exhaustive_through_length_four` and `escaped_unicode_round_trips` |
| P2 | `crates/web/src/lib.rs`, `render`, `page`, `card`, `when`, `tags`, `filters`, `coverage`, `facts`, `entry_page`, approach pages | `every_text_sink_rejects_the_attack_corpus` and `generated_text_cannot_change_html` |
| P2 | `crates/web/src/states.rs`, `pages`, `tile_map`, `index`, `cites`, `row`, `pathway`, `field_label`, `state_page` | The same P2 checks reach state labels, both pathway branches, duties, availability, facts, and citation tooltips. |
| P3 | `crates/web/src/lib.rs`, `href`, `primary_source`, `card`, `facts`, `entry_page`; `crates/web/src/states.rs`, `cites` | `external_links_reject_the_scheme_and_attack_cross_product` and `generated_external_links_are_http_or_neutralized` |

`crates/web/tests/html_safety.rs` owns the executable input inventory in `TEXT_SLOTS`. It covers 35 string positions, including the fallback publisher text for missing source IDs and the fallback label for an unknown state claim field. Entry IDs and approaches reach slug-based URLs. Pathway IDs reach quoted IDs and fragment links. The state fixture has two pathways so that the least-regulated-path explanation renders.

The `build` entry point validates before rendering. `validate_states` restricts state codes to `STATE_CODES` before `states::path` uses them in internal URLs. Arbitrary invalid state codes passed directly to the public `render` function are outside this check. Add coverage before changing that validation boundary.

Source type and source-level checked date do not reach HTML. Facets, ages, duty flags, regulation labels, and template constants have closed Rust types or fixed values. This check does not claim arbitrary-string coverage for them.

## Tool and bounds

Cargo runs the real Rust encoder and renderer. The check pins proptest `1.8.0` and scraper `0.24.0` in `crates/web/Cargo.toml`, with transitive versions in `Cargo.lock`. scraper uses html5ever to parse the emitted HTML independently of Mercury's encoder.

- P1 exhausts all 11,111 strings of length zero through four over the ten-character alphabet in its test. It also samples 256 Unicode strings of at most 64 scalar values. The Unicode round-trip property excludes NUL and carriage return because HTML parsing normalizes them. P2 and P3 still exercise those characters for safety.
- P2 crosses every one of 35 slots with all 11 payloads in `ATTACKS`. It also samples 256 slot-and-Unicode-string pairs, with at most 64 scalar values after the fixed `probe` prefix.
- P3 crosses 21 URL prefixes with all 11 payloads, with both cited-source and website-fallback cards. It also samples 256 prefix-and-Unicode-tail pairs of at most 64 tail scalar values, again in both card branches.
- Each generated property uses ChaCha with seed `0x4d455243555259`. The deterministic corpus always runs. Proptest shrinks generated failures. Failure persistence is disabled so that deliberate scratch defects do not become committed regressions. Retain real counterexamples as explicit cases when fixing them.
- The fixture has one resource, one state, two pathways, and one source. P2 compares parsed HTML structure rather than output filenames because `slug` intentionally changes filenames. Reference-changing slots use a `probe`-prefixed baseline so that missing-source and unknown-field fallbacks remain comparable.

These are bounded exhaustive and generated property tests, not a theorem about every Rust string or a proof of browser execution. Kani would need a separate treatment of unbounded `String`, serialization, and HTML parsing. TLA+ would model different code rather than verify this sequential renderer. The Cargo check is the lowest-cost check of the actual failure boundary.

The check excludes semantic truth of state-law claims, legal review, calendar-date validity, regulation classification, stylesheet safety, DOM changes made by `filter.js`, `dump.json` consumers, Markdown in `llms.txt`, path traversal through invalid state codes, dependency compromise, and browser-specific parser differences. It supplements the real-product verification skill.

## Exercise scratch breaks

Use a disposable worktree at the candidate commit. Do not mutate another engineer's checkout. Reuse the slice's existing worker for each check. Keep the tests and this skill unchanged while applying each break separately.

1. P1. In `esc`, replace the `<` branch's `out.push_str("&lt;")` with `out.push('<')`. Run the one command. The delimiter-language check must fail on `<`.
2. P2. Restore `esc`. In `states::state_page`, replace `esc(&s.summary)` with `s.summary.clone()`. Run the one command. The attack-corpus check must name `/states/0/summary` and reject the added element from the quote-breakout payload.
3. P3. Restore `state_page`. In `href`, replace the rejected-URL branch's `"#".into()` with `esc(url)`. Run the one command. The scheme cross-product must reject a non-HTTP URL instead of receiving `#`.
4. Restore the scratch production code and run the same command again. Require the original six passing tests, or only the real findings recorded before the scratch changes.
5. Remove only the disposable worktree. Retain the rejection logs and restored result with the PR evidence, not in the repository.

For drift discovered after an unchecked change, use `/maintain-formal-verification-skill` and this coverage record. Do not advance the sync commit merely because Cargo passed.

## Target selection

Public HTML injection has site-wide impact and an open-ended input alphabet. The existing escaping test covers selected fields and a few literal payloads. The old non-HTTP test checks only a website-fallback card with `javascript:`. Neither establishes structure preservation for source publishers, labels, citation attributes, every duty, or generated Unicode and URL combinations.

The close second target is state-law provenance validation. It can prevent unsupported facts from looking authoritative, but existing tests already reject a missing quote, wrong source type, guessed claim, malformed date, missing duty claim, and unknown field. They also exercise validation before building. Validation cannot establish that a quote supports the displayed fact, that a source labeled `statute` is a statute, or that legal advice is correct. Those semantic gaps need their own contract.

Regulation has seven Boolean duties and four ordered outcomes, so exhaustive coverage would be cheap. Its current tests already exercise minimum-path selection and the approval, assessment, notice, and teacher-qualification branches. A wrong badge harms comparison but is less consequential than script execution or unsourced state-law statements.

Foundation article 11 does not require a custom state machine for this static generator. Mercury has no costly ordering, concurrency, or durability promise in these functions. This bounded security property is additional evidence, not a claim that a transition-system proof applies.
