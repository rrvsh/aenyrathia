use clap::{Parser, Subcommand};
use std::io;

mod render;
mod roll;
mod scan;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rolls some dice
    Roll(roll::RollArgs),
    /// Renders a given markdown file to HTML
    Render(render::RenderArgs),
    /// Scans a directory and lists the files and folders within
    Scan(scan::ScanArgs),
}

pub fn run() -> io::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Roll(args) => args.run(),
        Commands::Render(args) => args.run()?,
        Commands::Scan(args) => args.run(),
    }
    Ok(())
}
