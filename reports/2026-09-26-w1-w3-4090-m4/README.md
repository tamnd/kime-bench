# W1 and W3 on an RTX 4090 and an Apple M4, kime 0.0.25 against Laya 0.3.20

The first measured run of this harness. kime and Laya read the same request files, `workloads/W1/requests.jsonl` and `workloads/W3/requests.jsonl`, on the same machine in the same session, one call at a time.

## Machines

The 4090 box, from `kime-bench machine`:

```
os: windows
arch: x86_64
cpu: unknown
threads: 32
memory: unknown
nvidia gpus: NVIDIA GeForce RTX 4090, 610.62
```

The Mac, from `kime-bench machine`, on macOS 15.8.1:

```
os: macos
arch: aarch64
cpu: Apple M4
threads: 10
memory: 24.0 GiB
nvidia gpus: none found
```

## Versions

- kime 0.0.25 from crates.io, as pinned in this repository's Cargo.toml, harness commit on branch `w1-w3-runners`.
- Laya 0.3.20, the wheel from its Hugging Face repository.
- Model: the Laya checkpoint, `model.safetensors` sha256 prefix `891102d372688fc2`, the same file on both machines.
- 4090: NVIDIA driver 610.62, CUDA 13.3 user mode driver, Windows. Laya runs on torch 2.11.0+cu128 under Python 3.12.
- M4: macOS 15.8.1, torch 2.14.0 under Python 3.12.

## Commands

kime, for each workload and precision:

```
kime-bench run workloads/W1/requests.jsonl --model <laya dir> --device cuda --precision f16 --out kime-W1-cuda-f16.tsv
```

Laya, for each workload and setting:

```
python runners/run_laya.py workloads/W1/requests.jsonl <laya dir> --device cuda [--half] --out laya-W1-cuda-fp32.tsv
```

On the 4090 every run is 200 warmup calls and 2,000 measured calls. On the M4 every run is 50 warmup calls and 300 measured calls, which is below the 2,000 the rules ask for, so the M4 rows are a first look and not a result.

## RTX 4090

| Workload | Engine | p50 ms | p95 ms | p99 ms | mean ms | Laya p50 over kime p50 |
|---|---|---|---|---|---|---|
| W1 | Laya fp32 | 35.72 | 36.37 | 36.77 | 35.72 | |
| W1 | Laya fp16 autocast | 35.84 | 36.62 | 37.40 | 35.91 | |
| W1 | kime f16 | 2.72 | 11.10 | 13.14 | 5.10 | 13.1x |
| W1 | kime f32 | 10.93 | 13.13 | 15.31 | 10.35 | 3.3x |
| W3 | Laya fp32 | 54.24 | 54.80 | 55.03 | 54.30 | |
| W3 | Laya fp16 autocast | 54.38 | 55.22 | 55.48 | 54.54 | |
| W3 | kime f16 | 55.21 | 60.52 | 65.62 | 55.00 | 0.98x |
| W3 | kime f32 | 151.62 | 159.81 | 163.53 | 152.20 | 0.36x |

W1 is one question over a 96 token state, 136 input tokens a call with the question. W3 is ten questions over a 512 token state, 5,120 input tokens a call, because both engines read the state once per question.

What this says:

- W1 median is 13.1x Laya with f16, but the p95 is only 3.3x and the p99 2.8x. kime's calls fall in two groups, most near 2.7 ms and about a fifth near 10 ms, and the slow ones come in runs in time rather than on particular requests. A second run of the same command gave p50 2.81 ms and p95 7.27 ms, and a third p50 5.31 ms. The tail is the thing to fix before this row counts as 10x, since the rules judge tails and not only medians.
- W3 is a tie with f16 and a loss with f32. Laya puts the ten questions in one padded batch, so it pays one launch sequence for 5,120 tokens, and at that size the work is mostly GEMMs where PyTorch's cuBLAS calls do well. For the compat family the state has to be read once per question, since the question and state tokens attend to each other from the first layer, so the 10x on W3 is planned for the native model that reads the state once (spec 05). The compat engine still has room here: 5,120 tokens on this model is about 2.2 TFLOP, which is 13 ms at the 4090's f16 peak.
- `torch.compile` could not run on this box. Laya's `compile=True` stops with `TritonMissing`, because Triton does not ship for Windows. So the Laya rows here are eager PyTorch, which is how Laya runs on Windows today, and a Linux run with compile is still owed.
- Laya's fp16 autocast rows give answers identical to its fp32 rows, and the same time, so autocast does not reach the part of Laya's graph that costs the time.

## Apple M4

| Workload | Engine | p50 ms | p95 ms | p99 ms | mean ms | Laya p50 over kime p50 |
|---|---|---|---|---|---|---|
| W1 | Laya mps fp32 | 83.44 | 98.75 | 120.25 | 84.31 | |
| W1 | Laya mps fp16 autocast | 104.17 | 179.34 | 216.91 | 114.99 | |
| W1 | kime metal f16 | 103.67 | 112.72 | 118.41 | 103.24 | 0.80x |
| W3 | Laya mps fp32 | 2971.97 | 3169.35 | 3334.21 | 2937.38 | |
| W3 | Laya mps fp16 autocast | 3437.85 | 3732.46 | 3992.70 | 3378.39 | |
| W3 | kime metal f16 | 3530.23 | 3769.79 | 3854.18 | 3274.74 | 0.84x |

kime on Metal is slower than Laya on MPS on both workloads. These runs shared the Mac with other work, a load average near 6, and a short Laya run earlier the same hour gave 61.5 ms at p50 on W1, so the absolute numbers move a lot, but the order held in every run. The Metal backend is at its first version, and the M4 milestone is where it gets the work.

## Answers

Every run writes the answers of its last pass over the requests next to the timings. Taking Laya on MPS in fp32 as the reference, since it matched kime to four decimals on earlier checks:

| Run | Same answer | Largest probability difference |
|---|---|---|
| kime cuda f32, W1 | 100 of 100 | 0.0001 |
| kime cuda f16, W1 | 100 of 100 | 0.0022 |
| Laya cuda fp32, W1 | 96 of 100 | 0.0173 |
| kime cuda f32, W3 | 495 of 500 | 0.0366 |
| kime cuda f16, W3 | 494 of 500 | 0.0364 |
| Laya cuda fp32, W3 | 494 of 500 | 0.0747 |

A choice counts as the same when the picked option matches, a score when its most likely level matches, and a yes or no when both sides of 0.5 match. kime on CUDA is closer to Laya's MPS answers than Laya on CUDA is. Laya's CUDA path lands 0.017 away on W1, which looks like reduced precision in its CUDA attention or matmuls, and it is worth a look on the Laya side.

## The box

The 4090 is shared. During these runs another process held about 17 GB of the card's memory, and 26 of the 29 `sshd` processes on the box were busy, which also made SSH drop connections. Neither was using the GPU's compute while idle (P8, 0 percent), but both were there during the runs and are a candidate for kime's slow group of calls. Laya's calls, which are longer, had a tight spread in the same session.

## Raw data

`raw/` has one Parquet file per run with the columns `call`, `request`, `input_tokens`, `questions` and `ns`, the same data as the TSV next to it, and the answers of each run as JSON lines where the runner wrote them. The kime Metal runs predate the answers file.
