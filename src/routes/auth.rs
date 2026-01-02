use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use askama::Template;
use axum::{
    Extension, Router,
    extract::{Form, Query},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
};
use log::error;
use serde::Deserialize;
use sqlx::PgPool;
use tower_cookies::{Cookie, Cookies};

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new()
            .route("/register", get(register_get).post(register_post))
            .route("/login", get(login_get).post(login_post))
            .route("/logout", post(logout_post))
    }
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    redirect_path: String,
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    redirect_to: Option<String>,
}

pub async fn register_get(Query(params): Query<RedirectQuery>) -> Result<Html<String>, StatusCode> {
    let redirect_path = params.redirect_to.unwrap_or_else(|| "/".to_string());

    RegisterTemplate { redirect_path }.render().map_or_else(
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
    Query(params): Query<RedirectQuery>,
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

    let redirect_path = params.redirect_to.unwrap_or_else(|| "/".to_string());

    Ok(Redirect::to(&redirect_path))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    redirect_path: String,
}

pub async fn login_get(Query(params): Query<RedirectQuery>) -> Result<Html<String>, StatusCode> {
    let redirect_path = params.redirect_to.unwrap_or_else(|| "/".to_string());

    LoginTemplate { redirect_path }.render().map_or_else(
        |e| {
            error!("Error rendering register template: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn login_post(
    db: Extension<PgPool>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, StatusCode> {
    let LoginForm { email, password } = form;

    let result = sqlx::query!(
        "select full_name, password_hash from user_data where email=$1",
        email
    )
    .fetch_one(&*db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let parsed_hash =
        PasswordHash::new(&result.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        cookies.add(Cookie::new("full_name", result.full_name));
        cookies.add(Cookie::new("email", email));
        let redirect_path = params.redirect_to.unwrap_or_else(|| "/".to_string());

        Ok(Redirect::to(&redirect_path))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn logout_post(cookies: Cookies, Query(params): Query<RedirectQuery>) -> Redirect {
    cookies.remove(Cookie::new("full_name", ""));
    cookies.remove(Cookie::new("email", ""));

    let redirect_path = params.redirect_to.unwrap_or_else(|| "/".to_string());

    Redirect::to(&redirect_path)
}
