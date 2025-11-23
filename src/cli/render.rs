use clap::Args;
use pinbreak::Markdown;
use std::io;

#[derive(Args)]
pub struct RenderArgs {
    // The path to the markdown file
    path: String,
}

impl RenderArgs {
    pub fn run(&self) -> io::Result<()> {
        println!("{}", Markdown::from_path(&self.path)?.as_html());
        Ok(())
    }
}
