use clap::Args;
use markdown::to_html;
use std::fs;

#[derive(Args)]
pub struct RenderArgs {
    // The path to the markdown file
    path: String,
}

impl RenderArgs {
    pub fn run(&self) {
        let read_result = fs::read_to_string(&self.path);
        match read_result {
            Ok(content) => println!("{}", to_html(&content)),
            Err(e) => println!("{e}"),
        }
    }
}
