use crate::formatting::normalise_newlines;
use crate::wiki::Wiki;
use askama::Template;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use log::error;
use markdown::to_html;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
}

pub async fn login(cookies: Cookies, Form(form): Form<LoginRequest>) -> Redirect {
    cookies.add(Cookie::new("username", form.username));
    Redirect::to("/")
}

pub async fn logout(cookies: Cookies) -> Redirect {
    cookies.remove(Cookie::new("username", ""));
    Redirect::to("/")
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    username: Option<String>,
    edit_mode: bool,
    raw_file_content: String,
    rendered_html: String,
}

#[derive(Deserialize)]
pub struct EditModeQuery {
    edit_mode: Option<bool>,
}

pub async fn render_wiki_page(
    cookies: Cookies,
    Path(article_path): Path<String>,
    Query(params): Query<EditModeQuery>,
    State(wiki): State<Wiki>,
) -> Result<Html<String>, StatusCode> {
    let branch_name = if params.edit_mode.unwrap_or(false) {
        cookies.get("username").map_or_else(
            || "prime".to_string(),
            |username| format!("user/{}", username.value()),
        )
    } else {
        "prime".to_string()
    };
    let file_content = wiki
        .get_remote_branch_file_contents(&article_path, &branch_name)
        .map_or_else(String::new, |file_content| file_content);
    ArticleTemplate {
        edit_mode: params.edit_mode.unwrap_or(false),
        username: cookies.get("username").map(|c| c.value().to_string()),
        raw_file_content: file_content.clone(),
        rendered_html: to_html(&file_content),
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering template for {article_path}: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct EditForm {
    markdown: String,
}

pub async fn update_wiki_page(
    Path(article_path): Path<String>,
    State(wiki): State<Wiki>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    if let Some(username) = cookies.get("username") {
        let git_branch = format!("user/{}", username.value());
        match wiki.update_remote_branch_file_contents(
            &article_path,
            &normalise_newlines(&form.markdown),
            &git_branch,
        ) {
            Ok(()) => Ok(Redirect::to(&("/".to_owned() + &article_path))),
            Err(()) => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Ok(Redirect::to(&("/".to_owned() + &article_path)))
    }
}
