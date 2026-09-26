# kime-bench

The benchmark harness for [kime](https://github.com/tamnd/kime).

Latency, throughput, energy, cost and accuracy for kime against Laya, laya-mlx, laya-coreml and Jev, on the same hardware in the same session, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is ten times faster at equal or better accuracy, which means its credibility rests on these measurements more than on any kernel. A benchmark number without its methodology is marketing.

The design is [`spec/13-benchmarks.md`](https://github.com/tamnd/kime/blob/main/spec/13-benchmarks.md) in the kime repository.

## Where kime is, today

The first run is in [reports/2026-09-26-w1-w3-4090-m4](reports/2026-09-26-w1-w3-4090-m4/README.md), kime 0.0.25 against Laya 0.3.20 on the same machine.

| Row | Laya | kime | Laya over kime |
|---|---|---|---|
| W1 on an RTX 4090, p50 | 35.72 ms | 2.72 ms (f16) | 13.1x |
| W1 on an RTX 4090, p95 | 36.37 ms | 11.10 ms (f16) | 3.3x, misses 10x on the tail |
| W3 on an RTX 4090, p50 | 54.24 ms | 55.21 ms (f16) | 0.98x, a tie |
| W1 on an Apple M4, p50 | 83.44 ms | 103.67 ms (Metal f16) | 0.80x, a loss |

W1 on the 4090 is past 10x at the median and not in the tail. W3 needs the state read once, which the compat family cannot do, and Metal needs the M4 milestone's work. The T4 rows from the specification are not run because there is no T4 here.
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
