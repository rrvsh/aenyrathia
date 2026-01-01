/// Replaces all occurences of `\r\n` and `\r` with `\n`.
pub fn normalise_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Resolves `{article_path}` to `wiki/{article_path}.md`.
/// Defaults to `index` when `article_path` is `None` or empty.
pub fn resolve_article_path(article_path: Option<String>) -> String {
    let ensured_article_path = match article_path {
        Some(article_path) if !article_path.is_empty() => article_path,
        _ => "index".to_string(),
    };
    String::from("wiki/") + &ensured_article_path + ".md"
}

/// Resolves branch name based on if user is logged in and in edit mode or not.
/// Defaults to `prime` when `edit_mode` is `None` or false or `full_name` is None.
pub fn resolve_branch_name(edit_mode: Option<bool>, full_name: Option<&String>) -> String {
    if edit_mode.unwrap_or(false) {
        full_name.map_or_else(
            || "prime".to_string(),
            |full_name| {
                let mut s = full_name
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                    .collect::<String>();

                while s.contains("--") {
                    s = s.replace("--", "-");
                }

                let full_name = s.trim_matches('-');

                if full_name.is_empty() {
                    "prime".to_string()
                } else {
                    format!("user/{full_name}")
                }
            },
        )
    } else {
        "prime".to_string()
    }
}
