use askama::{Result, Values};
use markdown::{CompileOptions, Options};

pub fn render_markdown(markdown: &str) -> String {
    markdown::to_html_with_options(
        markdown,
        &Options {
            compile: CompileOptions {
                allow_dangerous_html: true,
                ..CompileOptions::default()
            },
            ..Options::default()
        },
    )
    .unwrap_or_else(|_| markdown::to_html(markdown))
}

#[allow(clippy::unnecessary_wraps)]
/// Convert Markdown input to HTML so templates can render it directly.
pub fn html(markdown: &str, _: &dyn Values) -> Result<String> {
    Ok(render_markdown(markdown))
}
