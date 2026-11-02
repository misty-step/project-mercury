# Project Mercury

A sourced catalogue of learning resources. Every claim cites a source and says
how it was established: `measured` (checked on a date), `inferred`, or `guess`.

```sh
cargo run -p project-mercury -- search "reading age 5" --json
```

Data lives in `data/entries/*.yaml` and `data/sources/*.yaml`, one record per file.
Loading fails if a claim has no source, cites a source that doesn't exist, or is
`measured` without a `checked_on` date.

Licensed under MIT or Apache-2.0.
