use std::path::Path;
use std::process::ExitCode;

use kime_bench::machine::Machine;
use kime_bench::runner::{Options, run};
use kime_bench::stats::Calls;
use kime_bench::table::Table;

const HELP: &str = "kime-bench: the benchmark harness for kime

usage: kime-bench <command>
       kime-bench run <requests.jsonl> [--model laya] [--device auto|cpu|cuda|metal]
                  [--precision f16|f32|int8] [--warmup 200] [--calls 2000] [--out calls.tsv]
       kime-bench summary <calls.tsv>...

commands:
  workloads   print the workloads from spec/13-benchmarks.md
  baselines   print the published baseline for every speed and cost row, with its source
  machine     print what this machine is, which heads every report
  run         time kime in process on a workload, one call at a time
  summary     print p50, p95 and p99 of per call timings from any runner
  help        print this
";

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("workloads") => {
            Table::load(&root.join("workloads/workloads.tsv")).map(|t| t.markdown())
        }
        Some("baselines") => {
            Table::load(&root.join("baselines/published.tsv")).map(|t| t.markdown())
        }
        Some("machine") => Ok(Machine::detect().describe()),
        Some("run") => Options::parse(&args[1..]).and_then(|o| run(&o)),
        Some("summary") => args[1..]
            .iter()
            .map(|p| Calls::load(Path::new(p)).map(|c| c.summary(p)))
            .collect::<Result<Vec<_>, _>>()
            .map(|s| s.concat()),
        Some("--version" | "-V") => Ok(format!("kime-bench {}\n", env!("CARGO_PKG_VERSION"))),
        None | Some("help" | "--help" | "-h") => Ok(HELP.to_string()),
        Some(other) => Err(format!("unknown command {other}\n\n{HELP}")),
    };
    match result {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
