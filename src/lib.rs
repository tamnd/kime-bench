//! The benchmark harness for kime.
//!
//! The design is `spec/13-benchmarks.md` in the kime repository. At M0 this crate knows the
//! workloads, the published baselines and the machine it runs on. The runners that time kime and
//! Laya in the same session arrive with the M0 gate issue in tamnd/kime.

pub mod machine;
pub mod table;
