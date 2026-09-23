//! What machine this is. `spec/13-benchmarks.md` says a number without a machine attached cannot be
//! acted on, so every report starts with this.

use std::process::Command;

/// The facts about the host that go at the top of every report.
#[derive(Debug, Clone)]
pub struct Machine {
    pub os: String,
    pub arch: String,
    pub cpu: String,
    pub threads: usize,
    pub memory_bytes: Option<u64>,
    pub gpus: Vec<String>,
}

impl Machine {
    /// Reads the machine description from the operating system. Anything it cannot find is
    /// reported as unknown rather than guessed.
    pub fn detect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu: cpu_name().unwrap_or_else(|| "unknown".into()),
            threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
            memory_bytes: memory_bytes(),
            gpus: nvidia_gpus(),
        }
    }

    /// The description as `key: value` lines.
    pub fn describe(&self) -> String {
        let memory = self.memory_bytes.map_or_else(
            || "unknown".into(),
            |b| format!("{:.1} GiB", b as f64 / (1u64 << 30) as f64),
        );
        let gpus = if self.gpus.is_empty() { "none found".into() } else { self.gpus.join(", ") };
        format!(
            "os: {}\narch: {}\ncpu: {}\nthreads: {}\nmemory: {}\nnvidia gpus: {}\n",
            self.os, self.arch, self.cpu, self.threads, memory, gpus
        )
    }
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn cpu_name() -> Option<String> {
    if cfg!(target_os = "macos") {
        return run("sysctl", &["-n", "machdep.cpu.brand_string"]);
    }
    let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    info.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
}

fn memory_bytes() -> Option<u64> {
    if cfg!(target_os = "macos") {
        return run("sysctl", &["-n", "hw.memsize"])?.parse().ok();
    }
    let info = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kib: u64 = info
        .lines()
        .find(|l| l.starts_with("MemTotal:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    Some(kib * 1024)
}

fn nvidia_gpus() -> Vec<String> {
    run("nvidia-smi", &["--query-gpu=name,driver_version", "--format=csv,noheader"])
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}
