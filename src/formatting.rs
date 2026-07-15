pub const HOME_PAGE: &str = "Home";

/// Replaces all occurences of `\r\n` and `\r` with `\n`.
pub fn normalise_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidWikiPath;

/// Resolves `{article_path}` to `wiki/{article_path}.md`.
/// Defaults to `Home` when `article_path` is `None` or empty.
pub fn resolve_article_path(article_path: Option<String>) -> Result<String, InvalidWikiPath> {
    let ensured_article_path = resolve_article_slug(article_path)?;
    Ok(String::from("wiki/") + &ensured_article_path + ".md")
}

/// Resolves `{article_path}` to a validated wiki slug without the `wiki/` prefix or `.md` suffix.
pub fn resolve_article_slug(article_path: Option<String>) -> Result<String, InvalidWikiPath> {
    let Some(article_path) = article_path else {
        return Ok(HOME_PAGE.to_string());
    };

    if article_path.is_empty() {
        return Ok(HOME_PAGE.to_string());
    }

    validate_wiki_slug(&article_path)?;
    Ok(article_path)
}

fn validate_wiki_slug(article_path: &str) -> Result<(), InvalidWikiPath> {
    if article_path.starts_with('/')
        || article_path.ends_with('/')
        || article_path.contains('\\')
        || article_path.contains('\0')
        || article_path.chars().any(char::is_control)
    {
        return Err(InvalidWikiPath);
    }

    for segment in article_path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(InvalidWikiPath);
        }
    }

    Ok(())
}

/// Resolves branch name based on if user is logged in and in edit mode or not.
/// Defaults to `prime` when `edit_mode` is `None` or false or `email` is None.
pub fn resolve_branch_name(edit_mode: Option<bool>, email: Option<&String>) -> String {
    if edit_mode.unwrap_or(false) {
        email.map_or_else(
            || "prime".to_string(),
            |email| {
                let mut s = email
                    .trim()
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                    .collect::<String>();

                while s.contains("--") {
                    s = s.replace("--", "-");
                }

                let email = s.trim_matches('-');

                if email.is_empty() {
                    "prime".to_string()
                } else {
                    format!("user/{email}")
                }
            },
        )
    } else {
        "prime".to_string()
    }
}
