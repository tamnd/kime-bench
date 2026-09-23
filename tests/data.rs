//! The committed data says what it claims to say.

use std::path::Path;

use kime_bench::table::Table;

fn load(rel: &str) -> Table {
    Table::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("committed data parses")
}

#[test]
fn the_workloads_are_w1_to_w9_in_order() {
    let t = load("workloads/workloads.tsv");
    let ids: Vec<_> = t.values("id").unwrap().collect();
    let want: Vec<String> = (1..=9).map(|i| format!("W{i}")).collect();
    assert_eq!(ids, want);
}

#[test]
fn every_baseline_has_a_source_and_a_known_workload() {
    let workloads = load("workloads/workloads.tsv");
    let known: Vec<_> = workloads.values("id").unwrap().collect();
    let t = load("baselines/published.tsv");
    let (w, v, u, s) = (
        t.column("workload").unwrap(),
        t.column("value").unwrap(),
        t.column("unit").unwrap(),
        t.column("source").unwrap(),
    );
    for row in &t.rows {
        assert!(known.contains(&row[w].as_str()), "{}: unknown workload {}", row[0], row[w]);
        assert!(!row[s].trim().is_empty(), "{}: a baseline without a source", row[0]);
        assert!(
            ["ms", "J", "qps", "usd"].contains(&row[u].as_str()),
            "{}: unit {}",
            row[0],
            row[u]
        );
        // A number or a dash, and a dash only when the source says it is measured on the day.
        if row[v] == "-" {
            assert_eq!(row[s], "measured on the day", "{}", row[0]);
        } else {
            let x: f64 =
                row[v].parse().unwrap_or_else(|_| panic!("{}: {} is not a number", row[0], row[v]));
            assert!(x > 0.0, "{}", row[0]);
        }
    }
}

#[test]
fn row_ids_are_unique() {
    let t = load("baselines/published.tsv");
    let mut ids: Vec<_> = t.values("row").unwrap().collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n);
}
