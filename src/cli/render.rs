use clap::Args;
use markdown::to_html;
use std::fs;
use std::io;

#[derive(Args)]
pub struct RenderArgs {
    // The path to the markdown file
    path: String,
}

impl RenderArgs {
    pub fn run(&self) -> io::Result<()> {
        let content = fs::read_to_string(&self.path)?;
        println!("{}", to_html(&content));
        Ok(())
    }
}
