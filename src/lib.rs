//! The benchmark harness for kime.
//!
//! The design is `spec/13-benchmarks.md` in the kime repository. This crate knows the workloads,
//! the published baselines and the machine it runs on, times kime in process, and summarizes the
//! per call timings of kime and of Laya, whose runner is `runners/run_laya.py`.

pub mod machine;
pub mod runner;
pub mod stats;
pub mod table;
