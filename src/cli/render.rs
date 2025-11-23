use clap::Args;
use markdown::to_html;

#[derive(Args)]
pub struct RenderArgs {
    // The markdown content
    markdown: String,
}

impl RenderArgs {
    pub fn run(&self) {
        println!("{}", to_html(&self.markdown));
    }
}
