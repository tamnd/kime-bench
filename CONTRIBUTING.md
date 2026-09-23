# Contributing

The engine, its specification and most of the discussion live in [tamnd/kime](https://github.com/tamnd/kime). Issues about what to measure belong there, labelled `area/bench`. This repository is the apparatus.

## A new measurement

A report is a directory under `reports/`, named for the date and the run, with:

- the report itself as Markdown, starting with the output of `cargo run -- machine`,
- the exact command for each side, kime and the baseline,
- the versions: kime's git hash and model hash, the baseline's version and settings, the driver and the OS,
- the raw per call data as Parquet.

A report that is missing any of these is not merged, however good the number is. A report whose number misses the target is merged exactly like one that hits it.

## A new baseline

A row in `baselines/published.tsv` needs a source that someone else can check. "Measured on the day" is a source, and it means the harness measures the baseline in the same session as kime rather than reading a number from a file.

## Checks

```
cargo fmt --all --check
cargo clippy --all-targets --all-features
cargo test --all-features
```

The tests include the prose rules: plain English, no em dashes or en dashes, no horizontal rules, and no sentence broken across two lines.

## License

Apache-2.0, and a contribution is offered under the same terms.
