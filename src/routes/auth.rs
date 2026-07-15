use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
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
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use std::fmt::Write as _;
use tower_cookies::{Cookie, Cookies, cookie::SameSite};

const SESSION_COOKIE: &str = "session_id";
const CSRF_COOKIE: &str = "csrf_token";

pub struct AuthRouter {}

impl AuthRouter {
    pub fn build() -> Router {
        Router::new()
            .route("/register", get(register_get).post(register_post))
            .route("/login", get(login_get).post(login_post))
            .route("/logout", post(logout_post))
    }
}

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub full_name: String,
    pub email: String,
    pub csrf_token: String,
}

pub async fn current_user(db: Option<&SqlitePool>, cookies: &Cookies) -> Option<CurrentUser> {
    let db = db?;
    let token = cookies.get(SESSION_COOKIE)?.value().to_string();
    let row = sqlx::query(
        "select u.full_name, u.email, s.csrf_token \
         from user_session s \
         join user_data u on u.id = s.user_id \
         where s.token = ?",
    )
    .bind(token)
    .fetch_one(db)
    .await
    .ok()?;

    Some(CurrentUser {
        full_name: row.get("full_name"),
        email: row.get("email"),
        csrf_token: row.get("csrf_token"),
    })
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    redirect_path: String,
    csrf_token: String,
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    redirect_to: Option<String>,
}

pub fn safe_redirect_path(redirect_to: Option<String>) -> String {
    let Some(path) = redirect_to else {
        return "/".to_string();
    };

    if path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains(':')
        && !path.chars().any(char::is_control)
    {
        path
    } else {
        "/".to_string()
    }
}

pub async fn register_get(
    Extension(db): Extension<Option<SqlitePool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
) -> Result<Html<String>, StatusCode> {
    if db.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let redirect_path = encode_query_value(&safe_redirect_path(params.redirect_to));
    let csrf_token = set_csrf_cookie(&cookies);

    RegisterTemplate {
        redirect_path,
        csrf_token,
    }
    .render()
    .map_or_else(
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
    csrf_token: String,
}

pub async fn register_post(
    Extension(db): Extension<Option<SqlitePool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, StatusCode> {
    verify_csrf_cookie(&cookies, &form.csrf_token)?;
    let db = db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let fullname = form.fullname.trim().to_string();
    let email = form.email.trim().to_lowercase();
    let password = form.password;
    if fullname.is_empty() || email.is_empty() || password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("insert into user_data (full_name, email, password_hash) values (?, ?, ?)")
        .bind(fullname)
        .bind(email)
        .bind(password_hash.to_string())
        .execute(db)
        .await
        .map_err(|err| {
            if err
                .as_database_error()
                .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
            {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Redirect::to(&safe_redirect_path(params.redirect_to)))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    redirect_path: String,
    csrf_token: String,
}

pub async fn login_get(
    Extension(db): Extension<Option<SqlitePool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
) -> Result<Html<String>, StatusCode> {
    if db.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let redirect_path = encode_query_value(&safe_redirect_path(params.redirect_to));
    let csrf_token = set_csrf_cookie(&cookies);

    LoginTemplate {
        redirect_path,
        csrf_token,
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering login template: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
    csrf_token: String,
}

pub async fn login_post(
    Extension(db): Extension<Option<SqlitePool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, StatusCode> {
    verify_csrf_cookie(&cookies, &form.csrf_token)?;
    let db = db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let email = form.email.trim().to_lowercase();
    let password = form.password;
    if email.is_empty() || password.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let result = sqlx::query("select id, password_hash from user_data where email = ?")
        .bind(&email)
        .fetch_optional(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let user_id: i64 = result.get("id");
    let password_hash: String = result.get("password_hash");
    let parsed_hash =
        PasswordHash::new(&password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let session_token = random_token();
    let csrf_token = random_token();
    sqlx::query("insert into user_session (token, user_id, csrf_token) values (?, ?, ?)")
        .bind(&session_token)
        .bind(user_id)
        .bind(&csrf_token)
        .execute(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    cookies.add(session_cookie(&session_token));
    cookies.add(csrf_cookie(&csrf_token));

    Ok(Redirect::to(&safe_redirect_path(params.redirect_to)))
}

#[derive(Deserialize)]
pub struct CsrfForm {
    csrf_token: String,
}

pub async fn logout_post(
    Extension(db): Extension<Option<SqlitePool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<CsrfForm>,
) -> Result<Redirect, StatusCode> {
    verify_csrf_cookie(&cookies, &form.csrf_token)?;

    if let (Some(db), Some(session)) = (db.as_ref(), cookies.get(SESSION_COOKIE)) {
        let _ = sqlx::query("delete from user_session where token = ?")
            .bind(session.value())
            .execute(db)
            .await;
    }

    cookies.remove(removal_cookie(SESSION_COOKIE));
    cookies.remove(removal_cookie(CSRF_COOKIE));

    Ok(Redirect::to(&safe_redirect_path(params.redirect_to)))
}

pub fn verify_csrf_cookie(cookies: &Cookies, submitted: &str) -> Result<(), StatusCode> {
    let cookie = cookies
        .get(CSRF_COOKIE)
        .ok_or(StatusCode::FORBIDDEN)?
        .value()
        .to_string();

    if submitted.is_empty() || submitted != cookie {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}

fn set_csrf_cookie(cookies: &Cookies) -> String {
    let csrf_token = cookies
        .get(CSRF_COOKIE)
        .map_or_else(random_token, |cookie| cookie.value().to_string());
    cookies.add(csrf_cookie(&csrf_token));
    csrf_token
}

fn session_cookie(value: &str) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value.to_string()))
        .path("/")
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Lax)
        .build()
}

fn csrf_cookie(value: &str) -> Cookie<'static> {
    Cookie::build((CSRF_COOKIE, value.to_string()))
        .path("/")
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Lax)
        .build()
}

fn removal_cookie(name: &'static str) -> Cookie<'static> {
    Cookie::build(name).path("/").build()
}

fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE").is_ok_and(|value| value == "true" || value == "1")
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char);
            }
            _ => write!(encoded, "%{byte:02X}").expect("Error appending encoded byte."),
        }
    }
    encoded
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().fold(String::new(), |mut token, byte| {
        write!(token, "{byte:02x}").expect("Error appending token byte.");
        token
    })
}

#[cfg(test)]
mod tests {
    use super::AuthRouter;
    use axum::Router;
    use axum::body::Body;
    use axum::extract::Extension;
    use axum::http::{Method, Request, StatusCode};
    use sqlx::SqlitePool;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use tower::util::ServiceExt as _;
    use tower_cookies::CookieManagerLayer;

    fn build_request(method: Method, uri: &str, body: Body, is_form: bool) -> Request<Body> {
        let mut builder = Request::builder().method(method).uri(uri);
        if is_form {
            builder = builder.header("content-type", "application/x-www-form-urlencoded");
        }

        match builder.body(body) {
            Ok(request) => request,
            Err(error) => panic!("failed to build request: {error}"),
        }
    }

    fn lazy_pool() -> SqlitePool {
        SqlitePoolOptions::new().connect_lazy_with(SqliteConnectOptions::new())
    }

    fn auth_app(db: Option<SqlitePool>) -> Router {
        AuthRouter::build()
            .layer(CookieManagerLayer::new())
            .layer(Extension(db))
    }

    #[tokio::test]
    async fn register_and_login_get_return_service_unavailable_without_database() {
        let app = auth_app(None);

        let register_response = match app
            .clone()
            .oneshot(build_request(
                Method::GET,
                "/register",
                Body::empty(),
                false,
            ))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };
        let login_response = match app
            .oneshot(build_request(Method::GET, "/login", Body::empty(), false))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };

        assert_eq!(register_response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(login_response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn register_and_login_get_render_normally_with_database_pool_present() {
        let app = auth_app(Some(lazy_pool()));

        let register_response = match app
            .clone()
            .oneshot(build_request(
                Method::GET,
                "/register",
                Body::empty(),
                false,
            ))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };
        let login_response = match app
            .oneshot(build_request(Method::GET, "/login", Body::empty(), false))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };

        assert_eq!(register_response.status(), StatusCode::OK);
        assert_eq!(login_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn register_and_login_post_return_service_unavailable_without_database() {
        let app = auth_app(None);

        let register_body =
            Body::from("fullname=Test+User&email=test%40example.com&password=test&csrf_token=test");
        let register_response = match app
            .clone()
            .oneshot(build_request(
                Method::POST,
                "/register",
                register_body,
                true,
            ))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };

        let login_body = Body::from("email=test%40example.com&password=test&csrf_token=test");
        let login_response = match app
            .oneshot(build_request(Method::POST, "/login", login_body, true))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };

        assert_eq!(register_response.status(), StatusCode::FORBIDDEN);
        assert_eq!(login_response.status(), StatusCode::FORBIDDEN);
    }
}
