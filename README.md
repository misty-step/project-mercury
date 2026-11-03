# Project Mercury

A navigable, sourced catalog of favorite homeschool resources, written in Rust.
Every factual claim in the catalog cites a source and says how it was obtained.

## Layout

- `data/entries/*.yaml`: one catalog entry per file
- `data/sources/*.yaml`: a list of cited sources per file (usually one file per publisher)
- `crates/core` (`project-mercury-core`): types, YAML loading, `validate()`, and a pure `search()`
- `crates/cli` (`project-mercury-cli`, binary `project-mercury`): command-line interface

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

`search [QUERY] [--subject S] [--age N] [--approach A] [--json]` matches QUERY
case-insensitively against name, summary and subjects. `--data DIR` overrides the
default `./data`.

## Licence

MIT, for both code and data. See [LICENSE-MIT](LICENSE-MIT).
