use std::path::Path;
use std::process::ExitCode;

use kime_bench::machine::Machine;
use kime_bench::table::Table;

const HELP: &str = "kime-bench: the benchmark harness for kime

usage: kime-bench <command>

commands:
  workloads   print the workloads from spec/13-benchmarks.md
  baselines   print the published baseline for every speed and cost row, with its source
  machine     print what this machine is, which heads every report
  help        print this
";

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let command = std::env::args().nth(1);
    let result = match command.as_deref() {
        Some("workloads") => {
            Table::load(&root.join("workloads/workloads.tsv")).map(|t| t.markdown())
        }
        Some("baselines") => {
            Table::load(&root.join("baselines/published.tsv")).map(|t| t.markdown())
        }
        Some("machine") => Ok(Machine::detect().describe()),
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
