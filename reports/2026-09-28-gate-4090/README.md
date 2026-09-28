# The M0 gate on an RTX 4090, kime 0.1.0 against Laya 0.3.20

This is the run for kime #19, the gate that closes M0. It asks for W1 at 12 ms or less at p50 and W3 at 2 ms or less per question on the 4090, with the ratio to Laya on the same card in the same session. W1 passes. W3 misses at 3.79 ms per question, and the ratio to Laya is 8.1x on W1 and 1.45x on W3, short of the 10x goal on both. The report is published with the miss, as the issue says.

## Machine and versions

- The shared RTX 4090 box from the earlier reports, Windows, NVIDIA driver 610.62, power limit 450 W, max SM clock 3135 MHz.
- kime 0.1.0 from crates.io, harness commit 2ab2571 of this repository.
- Laya 0.3.20 on torch 2.11.0+cu128 (CUDA 12.8) under Python 3.12. The issue names Laya 0.3.7, but 0.3.20 is the wheel on its Hugging Face repository today and the one all earlier reports used.
- Model: the Laya checkpoint, `model.safetensors` sha256 `891102d372688fc2a094dac56a384bc537b87c63f21f9f3dac0be2b7cbc8d86c`, the same file for both engines.
- All eight runs below ran back to back in one session on 2026-09-28, with nothing else of ours on the box.

## Commands

```
python runners/run_laya.py workloads/W1/requests.jsonl <laya dir> --device cuda --out laya-W1-cuda-fp32.tsv
python runners/run_laya.py workloads/W1/requests.jsonl <laya dir> --device cuda --half --out laya-W1-cuda-half.tsv
kime-bench run workloads/W1/requests.jsonl --model <laya dir> --device cuda --precision f16 --out kime-W1-cuda-f16.tsv
kime-bench run workloads/W1/requests.jsonl --model <laya dir> --device cuda --precision f32 --out kime-W1-cuda-f32.tsv
```

The same four for W3. Every run is 200 warmup calls and 2,000 measured calls. Laya's `--compile` could not run, since Triton does not ship for Windows, so Laya's best here is eager PyTorch with the Agent preloaded, in fp32 or under fp16 autocast.

## Results

| Workload | Engine | p50 ms | p95 ms | p99 ms | mean ms |
|---|---|---|---|---|---|
| W1 | Laya fp32 | 35.792 | 36.721 | 37.228 | 35.839 |
| W1 | Laya fp16 autocast | 35.244 | 36.541 | 38.140 | 35.434 |
| W1 | kime f16 | 4.343 | 6.213 | 7.832 | 4.156 |
| W1 | kime f32 | 6.087 | 9.504 | 10.500 | 6.729 |
| W3 | Laya fp32 | 54.808 | 55.361 | 57.306 | 54.873 |
| W3 | Laya fp16 autocast | 55.002 | 55.394 | 56.039 | 55.022 |
| W3 | kime f16 | 37.905 | 41.713 | 43.101 | 38.402 |
| W3 | kime f32 | 148.813 | 152.292 | 153.638 | 149.190 |

Per question on W3, which has ten questions a call: Laya fp32 5.481 ms at p50, kime f16 3.791 ms at p50, 4.171 ms at p95 and 4.310 ms at p99.

Against Laya's best setting on each workload, with kime f16:

| Workload | p50 | p95 | p99 |
|---|---|---|---|
| W1, against Laya fp16 autocast | 8.1x | 5.9x | 4.9x |
| W3, against Laya fp32 | 1.45x | 1.33x | 1.33x |

## The gate

| Target | Measured | Result |
|---|---|---|
| W1 p50 12 ms or less | 4.343 ms | pass |
| W3 2 ms or less per question | 3.791 ms | miss |
| Ratio to Laya in the report | above | done |
| 10x Laya | 8.1x W1, 1.45x W3 | miss |

W3 is bound by GEMM work. Ten questions over a 512 token state are 5,120 tokens, about 3.77 TFLOP a call, which is 23 ms at the 4090's f16 peak with f32 accumulation, so 2 ms per question cannot be reached in f16 on this card with the compat model. The two ways left are FP8 weights (spec 08), which the 4090's tensor cores run at twice the f16 rate, and the native model that reads the state once and not once per question (spec 05).

## The W1 slow phase

W1 f16 has two speeds that come in blocks of time and not on particular requests: about 2.6 ms a call (p10 is 2.617 ms, 13.5x Laya) and about 4.8 ms. This run landed mostly in the slow one, so its p50 is 4.34 ms where the 0.1.0 report had 2.61 ms.

To find where the time goes, a build of kime with CUDA events around the graph launch ran W1 3,000 times. Medians in microseconds:

| Calls | Count | launch | wait for the stream | graph on device |
|---|---|---|---|---|
| fast | 893 | 109 | 2494 | 2584 |
| slow | 2107 | 120 | 4823 | 2588 |

The graph takes the same time on the device in both groups, and the host work around it is the same too. The extra 2.3 ms is spent before the GPU starts our graph. The clocks stayed at 2775 MHz in P2 in both phases (`raw/clocks-W1.csv`), while GPU utilization fell from about 97 percent to about 53 percent in the slow phase. The box is shared and another process holds about 17 GB of the card, so the likely cause is the driver time slicing the GPU between contexts under Windows. Waiting on the stream with a spin loop instead of `cuStreamSynchronize` did not change it (p50 5.19 ms against 5.15 ms in a pair of runs), so kime will keep the plain synchronize. A Linux box with the card to itself is the way to take this out of the numbers.

## Answers

Taking Laya fp32 on CUDA as the reference:

| Run | Same answer | Largest probability difference |
|---|---|---|
| Laya fp16 autocast, W1 | 100 of 100 | 0.0000 |
| kime f16, W1 | 96 of 100 | 0.0172 |
| kime f32, W1 | 96 of 100 | 0.0173 |
| Laya fp16 autocast, W3 | 500 of 500 | 0.0000 |
| kime f16, W3 | 498 of 500 | 0.0893 |
| kime f32, W3 | 497 of 500 | 0.0852 |

kime f16 and f32 agree with each other, and the W1 gap to Laya on CUDA is the one the first report found against Laya on MPS as well, where kime matched and Laya on CUDA did not.

## Raw data

`raw/` has the TSV and Parquet of each run with the columns `call`, `request`, `input_tokens`, `questions` and `ns`, the answers of the last pass as JSON lines, the event trace of the W1 slow phase run as `trace-W1-cuda-f16.tsv` (the first 200 rows are warmup), and the `nvidia-smi` samples taken every 100 ms during a W1 run as `clocks-W1.csv`.
