use askama::Template;
use axum::Router;
use axum::extract::Form;
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use log::error;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new()
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/register", get(register_get))
    }
}

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
#[template(path = "register.html")]
struct RegisterTemplate {}

pub async fn register_get() -> Result<Html<String>, StatusCode> {
    RegisterTemplate {}.render().map_or_else(
        |e| {
            error!("Error rendering register template: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}
