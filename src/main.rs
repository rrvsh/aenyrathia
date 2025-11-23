use std::io;
mod cli;

fn main() -> io::Result<()> {
    cli::run()
}
