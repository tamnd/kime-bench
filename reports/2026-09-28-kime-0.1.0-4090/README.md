# W1 and W3 on an RTX 4090, kime 0.1.0

kime 0.1.0 lets each CUDA bucket run a faster cuBLASLt GEMM algorithm, as long as it gives the same bits as the one ranked for 256 rows (kime #170). This report runs W1 and W3 again on the same 4090 as [the 0.0.26 report](../2026-09-28-kime-0.0.26-4090/README.md) to see what that bought. Laya was not run again, so the Laya numbers below are from [the first report](../2026-09-26-w1-w3-4090-m4/README.md), on the same box with the same model file.

## Machine and versions

- The shared RTX 4090 box from the earlier reports, Windows, NVIDIA driver 610.62.
- kime 0.1.0 from crates.io, as pinned in this branch's Cargo.toml.
- Model: the Laya checkpoint, `model.safetensors` sha256 prefix `891102d372688fc2`.
- For a same session control, kime 0.0.26 ran W1 and W3 on the box about ten minutes before 0.1.0.

## Commands

```
kime-bench run workloads/W1/requests.jsonl --model <laya dir> --device cuda --precision f16 --out kime-W1-cuda-f16.tsv
kime-bench run workloads/W3/requests.jsonl --model <laya dir> --device cuda --precision f16 --out kime-W3-cuda-f16.tsv
```

Every run is 200 warmup calls and 2,000 measured calls.

## Results

| Workload | Engine | p50 ms | p95 ms | p99 ms | mean ms | Laya p50 over kime p50 |
|---|---|---|---|---|---|---|
| W1 | Laya fp32 (first report) | 35.72 | 36.37 | 36.77 | 35.72 | |
| W1 | kime 0.0.26 f16, same session | 4.191 | 5.879 | 7.128 | 4.051 | 8.5x |
| W1 | kime 0.1.0 f16 | 2.609 | 5.255 | 6.065 | 3.405 | 13.7x |
| W3 | Laya fp32 (first report) | 54.24 | 54.80 | 55.03 | 54.30 | |
| W3 | kime 0.0.26 f16, same session | 43.514 | 47.380 | 50.098 | 44.311 | 1.25x |
| W3 | kime 0.1.0 f16 | 38.151 | 41.528 | 42.441 | 38.336 | 1.42x |

What this says:

- W3 is 12 percent faster than 0.0.26 in the same session, 43.5 ms to 38.2 ms at p50, and the p99 drops by 7.7 ms. Against Laya it is now 1.42x at p50 and 1.32x at p95. Per question that is 3.8 ms, against the 2 ms per question the gate in kime #19 asks for, and the rest of that gap is GEMM work that 10x on W3 needs the native model for (spec 05).
- W1 is 2.61 ms at p50 and 13.7x Laya. The 0.0.26 control landed at 4.19 ms because this run fell in the box's slow group of calls, which comes and goes between runs, so the W1 median change is mostly the box and not the release. At p10, where the slow group does not reach, 0.1.0 is at 2.60 ms. The p95 is 5.3 ms, 6.9x Laya, still short of 10x on the tail.

## Answers

The answers of both workloads are byte for byte the same as 0.0.26's in the earlier report, so the faster GEMM algorithms give the same bits, as kime #170 says they should. The comparison with Laya in the 0.0.26 report holds as it is: 100 of 100 same answers on W1 and 494 of 500 on W3.

## Raw data

`raw/` has the TSV of each run with the columns `call`, `request`, `input_tokens`, `questions` and `ns`, a Parquet file with the same data, and the answers of the last pass as JSON lines.
