use crate::app::state::AppState;
use crate::formatting::{normalise_newlines, resolve_article_path, resolve_branch_name};
use crate::routes::wiki::LoginState::{LoggedIn, LoggedOut, LoginForm, RegisterForm};
use askama::Template;
use axum::Router;
use axum::extract::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::get;
use log::error;
use markdown::to_html;
use serde::Deserialize;
use tower_cookies::Cookies;

pub struct WikiRouter {}

impl WikiRouter {
    pub fn build(state: AppState) -> Router {
        let handlers = get(article_get).post(article_post);
        Router::new()
            .route("/", handlers.clone())
            .route("/{*article_path}", handlers)
            .with_state(state)
    }
}

enum LoginState {
    LoggedOut,
    LoggedIn(String),
    LoginForm,
    RegisterForm,
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    login_state: LoginState,
    edit_mode: bool,
    raw_file_content: String,
    rendered_html: String,
    error_message: String,
    redirect_path: String,
}

#[derive(Deserialize)]
pub struct ArticlePageParams {
    edit_mode: Option<bool>,
    to_state: Option<String>,
    error: Option<String>,
}

pub async fn article_get(
    cookies: Cookies,
    article_path: Option<Path<String>>,
    Query(params): Query<ArticlePageParams>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let username = cookies
        .get("username")
        .map(|cookie| cookie.value().to_string());
    let redirect_path = String::from("/") + &article_path.clone().unwrap_or_default();
    let relative_path = resolve_article_path(article_path);
    let branch_name = resolve_branch_name(params.edit_mode, username.as_ref());
    let edit_mode = if username.is_none() {
        false
    } else {
        params.edit_mode.unwrap_or(false)
    };

    let ui_state = cookies
        .get("ui_state")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| params.to_state.clone());
    let mut login_state = LoginState::LoggedOut;
    if let Some(username) = cookies.get("username") {
        login_state = LoginState::LoggedIn(username.value().to_string());
    } else if let Some(state) = ui_state {
        login_state = match state.as_str() {
            "login_form" => LoginForm,
            "register_form" => RegisterForm,
            _ => LoggedOut,
        };
    }

    let file_content = state.remote.read_file(&relative_path, Some(&branch_name));
    let mut raw_file_content = String::new();
    let mut rendered_html = String::new();
    if let Some(file_content) = file_content {
        rendered_html = to_html(&file_content);
        raw_file_content = file_content;
    } else if !edit_mode {
        return Err(StatusCode::NOT_FOUND);
    }
    let error_message = params.error.unwrap_or_default();
    let redirect_path_for_log = redirect_path.clone();
    ArticleTemplate {
        login_state,
        edit_mode,
        raw_file_content,
        rendered_html,
        error_message,
        redirect_path,
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering template for {redirect_path_for_log}: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct EditForm {
    markdown: String,
}

pub async fn article_post(
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let redirect_path = String::from("/") + &article_path.clone().unwrap_or_default();
    if let Some(username) = cookies.get("username") {
        let relative_path = resolve_article_path(article_path);
        let branch_name = Some(format!("user/{}", username.value()));
        let content = normalise_newlines(&form.markdown);
        match state
            .remote
            .write_file(&relative_path, &content, branch_name.as_deref())
        {
            Ok(()) => Ok(Redirect::to(&redirect_path)),
            Err(()) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Ok(Redirect::to(&redirect_path))
    }
}
