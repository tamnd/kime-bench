"""Times Laya in process on a workload's requests, one call at a time, with the columns that
`kime-bench run` writes, so `kime-bench summary` reads both.

    python runners/run_laya.py <requests.jsonl> <laya model dir> [--device cuda|mps|cpu] [--half]
        [--compile] [--threads N] [--warmup 200] [--calls 2000] [--out calls.tsv]

`--half` runs the model under fp16 autocast on cuda and mps, and `--compile` passes compile=True
to the Agent, which is Laya's own torch.compile switch. The spec asks for Laya at its best
settings, so a report runs each that helps and keeps the fastest. The answers of the last pass
over the requests are written next to the timings, so the two engines can be compared request by
request.
"""

import argparse
import contextlib
import json
import time

import torch
from laya.agent import Agent


def main():
    p = argparse.ArgumentParser()
    p.add_argument("requests")
    p.add_argument("model")
    p.add_argument("--device", default="cuda" if torch.cuda.is_available() else "cpu")
    p.add_argument("--half", action="store_true")
    p.add_argument("--compile", action="store_true")
    p.add_argument("--threads", type=int, default=0)
    p.add_argument("--warmup", type=int, default=200)
    p.add_argument("--calls", type=int, default=2000)
    p.add_argument("--out", default="calls.tsv")
    a = p.parse_args()
    if a.threads:
        torch.set_num_threads(a.threads)
    reqs = [json.loads(line) for line in open(a.requests, encoding="utf-8") if line.strip()]
    t = time.perf_counter()
    agent = Agent(a.model, device=a.device, compile=a.compile)
    load_ms = (time.perf_counter() - t) * 1e3
    autocast = (
        torch.autocast(device_type=a.device, dtype=torch.float16)
        if a.half and a.device in ("cuda", "mps")
        else contextlib.nullcontext()
    )

    def sync():
        if a.device == "cuda":
            torch.cuda.synchronize()
        elif a.device == "mps":
            torch.mps.synchronize()

    lines = ["call\trequest\tinput_tokens\tquestions\tns"]
    answers = {}
    with torch.inference_mode(), autocast:
        for call in range(a.warmup + a.calls):
            at = call % len(reqs)
            r = reqs[at]
            t = time.perf_counter_ns()
            out = agent.system_one(r["state"], r["questions"])
            sync()
            ns = time.perf_counter_ns() - t
            answers[at] = out
            if call >= a.warmup:
                tokens = out.get("usage", {}).get("input_tokens", 0)
                lines.append(f"{call - a.warmup}\t{at}\t{tokens}\t{len(r['questions'])}\t{ns}")
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    with open(a.out.rsplit(".", 1)[0] + ".answers.jsonl", "w") as f:
        for at in sorted(answers):
            f.write(json.dumps({"request": at, **answers[at]}) + "\n")
    print(
        f"laya {getattr(__import__('laya'), '__version__', '?')} on {a.device}, half {a.half}, "
        f"compile {a.compile}, torch {torch.__version__}, loaded in {load_ms:.0f} ms, {a.warmup} warmup calls"
    )


if __name__ == "__main__":
    main()
