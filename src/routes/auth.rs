use axum::Router;
use axum::extract::Form;
use axum::response::Redirect;
use axum::routing::post;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new()
            .route("/login", post(login))
            .route("/logout", post(logout))
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
