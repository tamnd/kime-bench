# kime-bench

The benchmark harness for [kime](https://github.com/tamnd/kime).

Latency, throughput, energy, cost and accuracy for kime against Laya, laya-mlx, laya-coreml and Jev, on the same hardware in the same session, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is ten times faster at equal or better accuracy, which means its credibility rests on these measurements more than on any kernel. A benchmark number without its methodology is marketing.

The design is [`spec/13-benchmarks.md`](https://github.com/tamnd/kime/blob/main/spec/13-benchmarks.md) in the kime repository.

## Where kime is, today

Nothing measured yet. kime is at M0, which is the engine running Laya's own weights, and the first rows this harness has to produce are W1 and W3 on a T4 against Laya 0.3.7 on the same machine. That is the M0 gate. When it runs, the report goes under `reports/` and the numbers go here, including the ones that miss.

## What is here

| Path | What it is |
|---|---|
| `workloads/workloads.tsv` | W1 to W9 as the specification defines them. The generated inputs for each land under `workloads/<id>/`. |
| `baselines/published.tsv` | The best baseline for every speed and cost row, and where each number comes from. |
| `reports/` | One directory per measured run: the report, the command, and the raw per call data. |

```sh
cargo run -- workloads
cargo run -- baselines
cargo run -- machine
```

## The rules

These are the rules from the specification, repeated here because they apply to every number in this repository and to every number in an issue that cites it.

- **Same hardware, same inputs, same session.** Every comparison against Laya runs on the same machine with the same requests. Jev is hosted, so it is compared on end to end HTTP latency from a stated location, and against the numbers TypeSafe publishes.
- **Baselines at their best.** Laya with tuned threads, fp16 autocast, the Router preloaded and torch.compile where it helps. laya-mlx and laya-coreml on Apple silicon. If a baseline improves, the run is repeated.
- **Tails, not only medians.** p50, p95 and p99, from at least 2,000 measured calls after 200 warmup calls.
- **Quality on held out test splits only**, with the contamination checks passing.
- **Every number comes with** the command that produced it, the git hash, the model hash, the driver and OS versions, and the raw data as Parquet.
- **Misses stay in.** A row that does not reach its target stays in the table with the reason next to it. The multilingual memory row is the first example.

Nothing here is measured on a GitHub runner. A timing from a shared virtual machine is not a timing. CI checks that the harness builds and that the committed data is well formed, which is a different question from what the data says.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0. See [LICENSE-APACHE](LICENSE-APACHE).
