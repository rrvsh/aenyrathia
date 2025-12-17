use clap::{Parser, Subcommand};
use std::io;

mod render;
mod serve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Renders a given markdown file to HTML
    Render(render::RenderArgs),
    /// Serves the app
    Serve(serve::ServeArgs),
}

pub fn run() -> io::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Render(args) => args.run()?,
        Commands::Serve(_) => serve::ServeArgs::run(),
    }
    Ok(())
}
