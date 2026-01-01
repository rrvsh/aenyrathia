use askama::{Result, Values};

/// Convert Markdown input to HTML so templates can render it directly.
pub fn html(markdown: &str, _: &dyn Values) -> Result<String> {
    Ok(markdown::to_html(markdown))
}
