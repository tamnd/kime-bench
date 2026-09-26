"""Writes the W1 and W3 request files from spec/13-benchmarks.md.

    python workloads/gen.py <papluca train parquet> <laya tokenizer.json>

The states are English texts from the papluca language identification train split, joined in order
and cut at exactly 96 tokens for W1 and 512 for W3 by Laya's tokenizer, not counting its special
tokens. The output is the same on every run, so the committed files can be checked by running it
again. It needs the `tokenizers` package and the `duckdb` command.
"""

import json
import subprocess
import sys
from pathlib import Path

from tokenizers import Tokenizer

ROOT = Path(__file__).resolve().parent

W1_QUESTIONS = {
    "intent": {
        "type": "choice",
        "instructions": "What does the customer want?",
        "criteria": {
            "exchange": "send the correct item",
            "refund": "money back",
            "tracking": "where is my order",
            "complaint": "general complaint with no request",
        },
    }
}

W3_QUESTIONS = {
    "topic": {
        "type": "choice",
        "instructions": "What is the text mostly about?",
        "criteria": {
            "product": "a product or its quality",
            "service": "customer service or delivery",
            "media": "a book, film or music",
            "other": "anything else",
        },
    },
    "sentiment": {
        "type": "choice",
        "instructions": "What is the overall tone?",
        "criteria": ["positive", "neutral", "negative", "mixed"],
    },
    "audience": {
        "type": "choice",
        "instructions": "Who is the writer speaking to?",
        "criteria": {"buyers": "other buyers", "seller": "the seller or brand", "self": "nobody in particular"},
    },
    "language": {
        "type": "choice",
        "instructions": "Which register is the text written in?",
        "criteria": ["formal", "casual", "slang"],
    },
    "satisfaction": {
        "type": "score",
        "instructions": "How satisfied is the writer?",
        "criteria": ["very unhappy", "unhappy", "neutral", "happy", "very happy"],
    },
    "urgency": {
        "type": "score",
        "instructions": "How urgent is a reply?",
        "criteria": ["not urgent", "soon", "critical"],
    },
    "detail": {
        "type": "score",
        "instructions": "How detailed is the text?",
        "criteria": ["very short", "some detail", "a lot of detail"],
    },
    "recommend": {"type": "noul", "instructions": "Does the writer recommend what they bought?"},
    "returning": {"type": "noul", "instructions": "Does the writer want to return something?"},
    "repeat": {"type": "noul", "instructions": "Would the writer buy from here again?"},
}


def texts(parquet):
    sql = f"COPY (SELECT text FROM read_parquet('{parquet}', file_row_number = true) WHERE labels = 'en' ORDER BY file_row_number) TO '/dev/stdout' (FORMAT json)"
    out = subprocess.run(["duckdb", "-c", sql], check=True, capture_output=True, text=True).stdout
    return [json.loads(line)["text"].strip() for line in out.splitlines() if line.strip()]


def states(tok, pool, at, tokens, count):
    """`count` states of exactly `tokens` tokens, each from the next texts in the pool from `at`, and
    where the next unused text is."""
    out = []
    while len(out) < count:
        body = ""
        while True:
            body = (body + "\n\n" + pool[at]).strip()
            at += 1
            enc = tok.encode(body, add_special_tokens=False)
            if len(enc.ids) >= tokens:
                break
        cut = enc.offsets[tokens - 1][1]
        state = body[:cut]
        n = len(tok.encode(state, add_special_tokens=False).ids)
        if n == tokens:
            out.append(state)
    return out, at


def write(name, rows):
    d = ROOT / name
    d.mkdir(exist_ok=True)
    with open(d / "requests.jsonl", "w") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"{name}: {len(rows)} requests")


def main():
    pool = texts(sys.argv[1])
    tok = Tokenizer.from_file(sys.argv[2])
    w1, at = states(tok, pool, 0, 96, 100)
    write("W1", [{"state": s, "questions": W1_QUESTIONS} for s in w1])
    w3, _ = states(tok, pool, at, 512, 50)
    write("W3", [{"state": s, "questions": W3_QUESTIONS} for s in w3])


if __name__ == "__main__":
    main()
