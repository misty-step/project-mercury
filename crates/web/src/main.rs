use std::path::PathBuf;

use clap::Parser;

/// Generate the Project Mercury static website.
#[derive(Parser)]
#[command(name = "mercury-site", version)]
struct Args {
    /// Data directory containing entries/ and sources/.
    #[arg(long, default_value = "data")]
    data: PathBuf,
    /// Output directory (created if missing; files are overwritten).
    #[arg(long, default_value = "dist")]
    out: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let n = mercury_site::build(&args.data, &args.out)?;
    println!("wrote {n} files to {}", args.out.display());
    Ok(())
}
