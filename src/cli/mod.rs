use clap::{Parser, Subcommand};

mod roll;
mod render;

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
    /// Renders markdown to HTML
    Render(render::RenderArgs),
}

pub fn run() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Roll(args) => args.run(),
        Commands::Render(args) => args.run(),
    }
}
