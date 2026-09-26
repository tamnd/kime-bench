//! Times kime in process on a workload's requests, one call at a time.
//!
//! The requests are read and validated before the clock starts, so a timed call is what an
//! application pays for `Kime::decide`: tokenizing, the forward pass and building the answer. The
//! answer cache is off, as it is by default in the library, so every call runs on the device. The
//! requests go round in file order until the warmup and measured calls are done, and the answers of
//! the last pass are written next to the timings.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use kime::request::{Limits, Request, parse};
use kime::{Device, Kime, Precision};
use serde_json::Value;

use crate::stats::{Calls, HEADER};

/// The kime release this harness links, the one pinned in Cargo.toml.
pub const KIME: &str = "0.0.25";

/// What `kime-bench run` was asked to do.
#[derive(Debug, Clone)]
pub struct Options {
    pub requests: PathBuf,
    pub model: String,
    pub device: Device,
    pub precision: Precision,
    pub warmup: usize,
    pub calls: usize,
    pub out: PathBuf,
}

impl Options {
    /// Parses the arguments after `run`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut o = Self {
            requests: PathBuf::new(),
            model: "laya".into(),
            device: Device::Auto,
            precision: Precision::F16,
            warmup: 200,
            calls: 2000,
            out: PathBuf::from("calls.tsv"),
        };
        let mut it = args.iter();
        while let Some(a) = it.next() {
            let mut val = || it.next().cloned().ok_or_else(|| format!("{a} needs a value"));
            match a.as_str() {
                "--model" => o.model = val()?,
                "--device" => {
                    o.device = match val()?.as_str() {
                        "auto" => Device::Auto,
                        "cpu" => Device::Cpu { threads: 0 },
                        "cuda" => Device::Cuda(0),
                        "metal" => Device::Metal,
                        d => return Err(format!("unknown device {d}")),
                    }
                }
                "--precision" => {
                    o.precision = match val()?.as_str() {
                        "f16" => Precision::F16,
                        "f32" => Precision::F32,
                        "int8" => Precision::Int8,
                        p => return Err(format!("unknown precision {p}")),
                    }
                }
                "--warmup" => o.warmup = val()?.parse().map_err(|e| format!("--warmup: {e}"))?,
                "--calls" => o.calls = val()?.parse().map_err(|e| format!("--calls: {e}"))?,
                "--out" => o.out = val()?.into(),
                p if !p.starts_with('-') && o.requests.as_os_str().is_empty() => {
                    o.requests = p.into();
                }
                other => return Err(format!("unknown option {other}")),
            }
        }
        if o.requests.as_os_str().is_empty() {
            return Err("run needs a requests.jsonl file".into());
        }
        Ok(o)
    }
}

fn load(path: &Path) -> Result<Vec<(Request, u32)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).map_err(|e| format!("line {}: {e}", i + 1))?;
        let questions = v["questions"].as_object().map_or(0, serde_json::Map::len) as u32;
        let req = parse(&v, &Limits::LAYA).map_err(|p| format!("line {}: {p:?}", i + 1))?;
        out.push((req, questions));
    }
    if out.is_empty() {
        return Err(format!("{} has no requests", path.display()));
    }
    Ok(out)
}

/// Runs the options and returns the summary to print.
pub fn run(o: &Options) -> Result<String, String> {
    let reqs = load(&o.requests)?;
    let t = Instant::now();
    let kime = Kime::builder()
        .model(&o.model)
        .device(o.device)
        .precision(o.precision)
        .preload(true)
        .build()
        .map_err(|e| e.to_string())?;
    let load_ms = t.elapsed().as_secs_f64() * 1e3;
    let mut f = std::fs::File::create(&o.out).map_err(|e| format!("{}: {e}", o.out.display()))?;
    let mut calls = Calls::default();
    let mut lines = vec![HEADER.to_string()];
    let mut answers = vec![Value::Null; reqs.len()];
    for call in 0..o.warmup + o.calls {
        let at = call % reqs.len();
        let (req, questions) = &reqs[at];
        let t = Instant::now();
        let res = kime.decide(req).map_err(|e| e.to_string())?;
        let ns = t.elapsed().as_nanos() as u64;
        answers[at] = res.to_json();
        if call < o.warmup {
            continue;
        }
        let tokens = res.input_tokens as u32;
        lines.push(format!("{}\t{at}\t{tokens}\t{questions}\t{ns}", call - o.warmup));
        calls.ns.push(ns);
        calls.questions.push(*questions);
        calls.tokens.push(tokens);
    }
    lines.push(String::new());
    f.write_all(lines.join("\n").as_bytes()).map_err(|e| e.to_string())?;
    // The answers of the last pass over the requests, next to the timings, so the two engines can
    // be compared request by request.
    let path = o.out.with_extension("answers.jsonl");
    let mut text = String::new();
    for (at, a) in answers.into_iter().enumerate().filter(|(_, a)| !a.is_null()) {
        let mut row = serde_json::json!({ "request": at });
        if let (Some(r), Value::Object(a)) = (row.as_object_mut(), a) {
            r.extend(a);
        }
        text.push_str(&row.to_string());
        text.push('\n');
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    let head = format!(
        "kime {} with {} on {}, {:?}, loaded in {load_ms:.0} ms, {} warmup calls\n",
        KIME,
        kime.model_id(),
        kime.device(),
        o.precision,
        o.warmup
    );
    Ok(head + &calls.summary(&o.requests.display().to_string()))
}
