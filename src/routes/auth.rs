use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use askama::Template;
use axum::{
    Extension, Router,
    extract::Form,
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new().route("/register", get(register_get).post(register_post))
    }
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

pub async fn register_post(
    db: Extension<PgPool>,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, StatusCode> {
    let RegisterForm {
        fullname,
        email,
        password,
    } = form;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        "insert into user_data (full_name, email, password_hash) values ($1, $2, $3)",
        fullname,
        email,
        password_hash.to_string(),
    )
    .execute(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/"))
}
