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

pub async fn register_get(
    Extension(db): Extension<Option<PgPool>>,
    Query(params): Query<RedirectQuery>,
) -> Result<Html<String>, StatusCode> {
    if db.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

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
    Extension(db): Extension<Option<PgPool>>,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, StatusCode> {
    let db = db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

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
    .execute(db)
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

pub async fn login_get(
    Extension(db): Extension<Option<PgPool>>,
    Query(params): Query<RedirectQuery>,
) -> Result<Html<String>, StatusCode> {
    if db.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

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
    Extension(db): Extension<Option<PgPool>>,
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, StatusCode> {
    let db = db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let LoginForm { email, password } = form;

    let result = sqlx::query!(
        "select full_name, password_hash from user_data where email=$1",
        email
    )
    .fetch_one(db)
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

#[cfg(test)]
mod tests {
    use super::AuthRouter;
    use axum::Router;
    use axum::body::Body;
    use axum::extract::Extension;
    use axum::http::{Method, Request, StatusCode};
    use sqlx::PgPool;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
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

    fn lazy_pool() -> PgPool {
        let options = PgConnectOptions::new()
            .host("localhost")
            .port(1)
            .username("unused")
            .password("unused")
            .database("unused");
        PgPoolOptions::new().connect_lazy_with(options)
    }

    fn auth_app(db: Option<PgPool>) -> Router {
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

        let register_body = Body::from("fullname=Test+User&email=test%40example.com&password=test");
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

        let login_body = Body::from("email=test%40example.com&password=test");
        let login_response = match app
            .oneshot(build_request(Method::POST, "/login", login_body, true))
            .await
        {
            Ok(response) => response,
            Err(error) => panic!("request failed: {error}"),
        };

        assert_eq!(register_response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(login_response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
