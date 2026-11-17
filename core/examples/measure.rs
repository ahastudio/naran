//! Measures how fast the core reads. Usage: cargo run --release -p naran-core --example measure -- <repository> [commit count]
#![allow(clippy::print_stdout, reason = "printing the times is the whole job")]

use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let path: PathBuf = arguments.next().ok_or("give the repository path")?.into();
    let limit = arguments.next().and_then(|count| count.parse().ok());
    measure_summary(&path)?;
    measure_history(&path, limit.unwrap_or(1000))
}

fn measure_summary(path: &Path) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let summary = naran_core::summarize(path)?;
    println!(
        "summarize      {:>8.1?}  ({})",
        start.elapsed(),
        summary.name
    );
    Ok(())
}

fn measure_history(path: &Path, limit: usize) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let history = naran_core::history(path, limit, naran_core::Refs::ALL)?;
    let lanes = history.rows.iter().map(|row| row.lane).max().unwrap_or(0) + 1;
    println!(
        "history {limit:>7} {:>8.1?}  (commits {}, lanes {lanes})",
        start.elapsed(),
        history.commits.len()
    );
    measure_json(&history)
}

fn measure_json(history: &naran_core::History) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let kilobytes = serde_json::to_vec(history)?.len() / 1024;
    println!(
        "  JSON serialize  {:>8.1?}  ({kilobytes} KB)",
        start.elapsed()
    );
    Ok(())
}
