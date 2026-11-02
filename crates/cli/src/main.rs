use clap::{Parser, Subcommand};
use mercury_core::{Entry, Source};
use serde::Serialize;
use std::path::PathBuf;
use std::process::ExitCode;

/// Search a sourced catalogue of learning resources.
#[derive(Parser)]
#[command(name = "project-mercury", version)]
struct Cli {
    /// Directory containing entries/ and sources/.
    #[arg(long, global = true, default_value = "data")]
    data: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search entries by name, subject and age, e.g. "reading age 5".
    Search {
        query: String,
        /// Emit JSON instead of text.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct JsonHit<'a> {
    score: usize,
    entry: &'a Entry,
    sources: Vec<&'a Source>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let catalog = match mercury_core::load(&cli.data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(problems) = mercury_core::validate(&catalog) {
        for p in problems {
            eprintln!("invalid: {p}");
        }
        return ExitCode::FAILURE;
    }

    let Command::Search { query, json } = cli.command;
    let hits = mercury_core::search(&catalog.entries, &query);
    if json {
        let out: Vec<JsonHit> = hits
            .iter()
            .map(|h| {
                let mut sources: Vec<&Source> = Vec::new();
                for c in &h.entry.claims {
                    if let Some(s) = c.source.as_deref().and_then(|id| catalog.source(id)) {
                        if !sources.iter().any(|x| x.id == s.id) {
                            sources.push(s);
                        }
                    }
                }
                JsonHit {
                    score: h.score,
                    entry: h.entry,
                    sources,
                }
            })
            .collect();
        match serde_json::to_string_pretty(&out) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else if hits.is_empty() {
        println!("No matches for '{query}'.");
    } else {
        for h in hits {
            let e = h.entry;
            println!(
                "{} (ages {}-{}) - {}",
                e.name,
                e.ages.min,
                e.ages.max,
                e.subjects.join(", ")
            );
            println!("  {}", e.url);
            for c in &e.claims {
                let source = c.source.as_deref().unwrap_or("?");
                let checked = c
                    .checked_on
                    .as_deref()
                    .map(|d| format!(", {d}"))
                    .unwrap_or_default();
                println!("  - {} [{:?}: {source}{checked}]", c.text, c.provenance);
            }
        }
    }
    ExitCode::SUCCESS
}
