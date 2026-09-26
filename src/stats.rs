//! Per call timings, as every runner writes them, and the percentiles a report quotes.
//!
//! A runner writes one TSV line per measured call: the call number, the request's line in the
//! workload file, its input tokens, its questions and the wall time in nanoseconds. kime's runner
//! and Laya's write the same columns, so one summary reads both.

use std::fmt::Write as _;
use std::path::Path;

use crate::table::Table;

/// The header every runner writes.
pub const HEADER: &str = "call\trequest\tinput_tokens\tquestions\tns";

/// The measured calls of one run.
#[derive(Debug, Clone, Default)]
pub struct Calls {
    pub ns: Vec<u64>,
    pub questions: Vec<u32>,
    pub tokens: Vec<u32>,
}

impl Calls {
    /// Reads a runner's TSV.
    pub fn load(path: &Path) -> Result<Self, String> {
        let t = Table::load(path)?;
        let col = |name: &str| {
            t.header.iter().position(|h| h == name).ok_or_else(|| format!("no {name} column"))
        };
        let (ns, q, tok) = (col("ns")?, col("questions")?, col("input_tokens")?);
        let mut c = Self::default();
        for (i, r) in t.rows.iter().enumerate() {
            let bad = |e: std::num::ParseIntError| format!("row {}: {e}", i + 1);
            c.ns.push(r[ns].parse().map_err(bad)?);
            c.questions.push(r[q].parse().map_err(bad)?);
            c.tokens.push(r[tok].parse().map_err(bad)?);
        }
        Ok(c)
    }

    /// p50, p95, p99 and mean in milliseconds, per call.
    pub fn percentiles(&self) -> [f64; 4] {
        let mut v: Vec<f64> = self.ns.iter().map(|&n| n as f64 / 1e6).collect();
        v.sort_by(f64::total_cmp);
        let at = |p: f64| {
            if v.is_empty() {
                return f64::NAN;
            }
            // Nearest rank, so a percentile is always a call that happened.
            let i = ((p * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1;
            v[i]
        };
        let mean = v.iter().sum::<f64>() / v.len().max(1) as f64;
        [at(0.50), at(0.95), at(0.99), mean]
    }

    /// The summary block of a report.
    pub fn summary(&self, name: &str) -> String {
        let [p50, p95, p99, mean] = self.percentiles();
        let q = self.questions.iter().map(|&q| u64::from(q)).sum::<u64>() as f64
            / self.questions.len().max(1) as f64;
        let tok = self.tokens.iter().map(|&t| u64::from(t)).sum::<u64>() as f64
            / self.tokens.len().max(1) as f64;
        let mut s = String::new();
        let _ = writeln!(
            s,
            "{name}: {} calls, {q:.1} questions and {tok:.1} input tokens a call",
            self.ns.len()
        );
        let _ = writeln!(
            s,
            "  per call      p50 {p50:.3} ms  p95 {p95:.3} ms  p99 {p99:.3} ms  mean {mean:.3} ms"
        );
        if q > 1.0 {
            let _ = writeln!(
                s,
                "  per question  p50 {:.3} ms  p95 {:.3} ms  p99 {:.3} ms  mean {:.3} ms",
                p50 / q,
                p95 / q,
                p99 / q,
                mean / q
            );
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let c = Calls {
            ns: (1..=100).map(|i| i * 1_000_000).collect(),
            questions: vec![1; 100],
            tokens: vec![96; 100],
        };
        assert_eq!(c.percentiles(), [50.0, 95.0, 99.0, 50.5]);
    }
}
