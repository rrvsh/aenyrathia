use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use askama::Template;
use axum::{
    Router,
    extract::Form,
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
};
use log::error;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new()
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/register", get(register_get).post(register_post))
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

#[derive(Deserialize)]
pub struct RegisterForm {
    fullname: String,
    email: String,
    password: String,
}

pub async fn register_post(Form(form): Form<RegisterForm>) -> Result<Redirect, StatusCode> {
    let RegisterForm {
        fullname,
        email,
        password,
    } = form;

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_or(Err(StatusCode::INTERNAL_SERVER_ERROR), |password_hash| {
            log::info!("Full Name: {fullname}, Email: {email}, Password Hash: {password_hash}");
            Ok(Redirect::to("/"))
        })
}
