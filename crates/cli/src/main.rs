use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use project_mercury_core::{load, search, validate, CostTier, Entry, Format, Query, SkillLevel};

/// Browse the Project Mercury catalog of homeschool resources.
#[derive(Parser)]
#[command(name = "project-mercury", version)]
struct Cli {
    /// Data directory containing entries/ and sources/.
    #[arg(long, global = true, default_value = "data")]
    data: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Filter entries by free text, subject, age, approach and facets.
    Search {
        query: Option<String>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        age: Option<u8>,
        #[arg(long)]
        approach: Option<String>,
        /// beginner | intermediate | advanced
        #[arg(long)]
        skill_level: Option<SkillLevel>,
        /// online | in_person | hybrid
        #[arg(long)]
        format: Option<Format>,
        /// free | low | mid | high
        #[arg(long)]
        cost_tier: Option<CostTier>,
        #[arg(long)]
        json: bool,
    },
    /// Show one entry with its claims and sources.
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Check that the data is internally consistent.
    Validate,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let catalog = load(&cli.data)?;
    match cli.command {
        Command::Search {
            query,
            subject,
            age,
            approach,
            skill_level,
            format,
            cost_tier,
            json,
        } => {
            let q = Query {
                term: query,
                subject,
                age,
                approach,
                skill_level,
                format,
                cost_tier,
            };
            let hits = search(&catalog.entries, &q);
            if json {
                println!("{}", serde_json::to_string_pretty(&hits)?);
            } else {
                hits.iter().for_each(|e| println!("{}", line(e)));
            }
        }
        Command::Show { id, json } => {
            let Some(e) = catalog.entry(&id) else {
                bail!("no entry with id `{id}`")
            };
            if json {
                println!("{}", serde_json::to_string_pretty(e)?);
                return Ok(());
            }
            println!("{}\n{}\n\n{}\n", line(e), e.url, e.summary);
            println!("subjects:   {}", e.subjects.join(", "));
            println!("approaches: {}", e.approaches.join(", "));
            println!("cost:       {}\n\nclaims:", e.cost);
            for c in &e.claims {
                let url = catalog.source(&c.source_id).map_or("?", |s| s.url.as_str());
                let on = c.checked_on.as_deref().unwrap_or("unchecked");
                println!(
                    "  {}: {} [{:?}, {on}] {url}",
                    c.field, c.value, c.provenance
                );
            }
        }
        Command::Validate => {
            if let Err(errors) = validate(&catalog) {
                errors.iter().for_each(|e| eprintln!("{e}"));
                bail!("{} validation error(s)", errors.len());
            }
            let (n, s) = (catalog.entries.len(), catalog.states.len());
            println!(
                "ok: {n} entries, {s} states, {} sources",
                catalog.sources.len()
            );
        }
    }
    Ok(())
}

fn line(e: &Entry) -> String {
    format!(
        "{:<24} {:<26} ages {}-{}",
        e.id, e.name, e.ages.min, e.ages.max
    )
}
