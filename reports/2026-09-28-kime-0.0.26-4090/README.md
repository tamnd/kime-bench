# W1 and W3 on an RTX 4090, kime 0.0.26

kime 0.0.26 moves the CUDA attention in f16 plans to tensor cores (kime #168). This report runs W1 and W3 again on the same 4090 as [the first report](../2026-09-26-w1-w3-4090-m4/README.md) to see what that bought. Laya was not run again, so the Laya numbers below are the ones from that report, on the same box with the same model file.

## Machine and versions

- The shared RTX 4090 box from the first report, Windows, NVIDIA driver 610.62.
- kime 0.0.26 from crates.io, as pinned in this branch's Cargo.toml.
- Model: the Laya checkpoint, `model.safetensors` sha256 prefix `891102d372688fc2`.
- For a same session control, kime 0.0.25 ran W3 and W1 on the box right before 0.0.26.

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
| W1 | kime 0.0.25 f16, same session | 2.771 | | | | 12.9x |
| W1 | kime 0.0.26 f16 | 2.715 | 5.995 | 7.625 | 3.809 | 13.2x |
| W3 | Laya fp32 (first report) | 54.24 | 54.80 | 55.03 | 54.30 | |
| W3 | kime 0.0.25 f16, same session | 52.013 | | | | 1.04x |
| W3 | kime 0.0.26 f16 | 43.448 | 47.383 | 48.951 | 44.285 | 1.25x |

What this says:

- W3 is 16 percent faster than 0.0.25 in the same session, 52.0 ms to 43.4 ms, and now beats Laya by 1.25x instead of tying it. The attention was about 8 ms of the 51 ms of device time on this workload and is now under 1 ms, so what is left is almost all GEMM, about 36 ms. That is where the next change goes.
- W1 barely moves at the median, since 136 tokens is not enough work for attention to matter. The p95 is 6.0 ms here against 11.1 ms in the first report, but the first report's slow group of calls came and went between runs on this shared box, so the tail is not something this release changed.
- W1 at 13.2x is past 10x at the median. The p95 is 6.1x, still short of 10x on the tail.

## Answers

Taking Laya on MPS in fp32 as the reference, as in the first report:

| Run | Same answer | Largest probability difference |
|---|---|---|
| kime 0.0.25 cuda f16, W1 | 100 of 100 | 0.0022 |
| kime 0.0.26 cuda f16, W1 | 100 of 100 | 0.0025 |
| kime 0.0.25 cuda f16, W3 | 494 of 500 | 0.0364 |
| kime 0.0.26 cuda f16, W3 | 494 of 500 | 0.0389 |

The same answers as 0.0.25 on every request, with the largest difference moving by a few thousandths. The tensor core attention rounds differently from the old scalar kernel, and kime's own parity test against the f32 reference shows the same small change.

## Raw data

`raw/` has the TSV of each run with the columns `call`, `request`, `input_tokens`, `questions` and `ns`, a Parquet file with the same data, and the answers of the last pass as JSON lines.
