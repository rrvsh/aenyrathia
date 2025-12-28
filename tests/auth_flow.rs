use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;

use aenyrathia::routes::auth::AuthRouter;

fn router() -> Router {
    AuthRouter::build().layer(CookieManagerLayer::new())
}

async fn extract_set_cookie(resp: &axum::http::Response<Body>, name: &str) -> Option<String> {
    resp.headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let s = value.to_str().ok()?;
            s.split(';')
                .next()
                .and_then(|pair| pair.split_once('='))
                .and_then(|(k, v)| (k.trim() == name).then_some(v.to_string()))
        })
}

#[tokio::test]
async fn login_success_sets_cookie_and_redirects() {
    let app = router();
    let req = Request::post("/login")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("username=alice&password=passw0rd"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        resp.headers()
            .get(axum::http::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap(),
        "/"
    );
    let cookie = extract_set_cookie(&resp, "username").await;
    assert_eq!(cookie.as_deref(), Some("alice"));
}

#[tokio::test]
async fn login_clears_ui_state_cookie() {
    let app = router();
    let req = Request::post("/login")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .header(axum::http::header::COOKIE, "ui_state=register_form")
        .body(Body::from("username=alice&password=passw0rd"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    let cleared = extract_set_cookie(&resp, "ui_state").await;
    // Clearing sets an empty cookie with expiry.
    assert_eq!(cleared.as_deref(), Some(""));
}

#[tokio::test]
async fn login_missing_fields_redirects_with_error() {
    let app = router();
    let req = Request::post("/login")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("username=alice&password="))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp
        .headers()
        .get(axum::http::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.contains("error=missing_fields"));
}

#[tokio::test]
async fn register_success_sets_cookie_and_redirects() {
    let app = router();
    let body = "fullname=Alice+Smith&email=a%40b.com&username=alice&password=passw0rd";
    let req = Request::post("/register")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let cookie = extract_set_cookie(&resp, "username").await;
    assert_eq!(cookie.as_deref(), Some("alice"));
}

#[tokio::test]
async fn register_clears_ui_state_cookie() {
    let app = router();
    let body = "fullname=Alice+Smith&email=a%40b.com&username=alice&password=passw0rd";
    let req = Request::post("/register")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .header(axum::http::header::COOKIE, "ui_state=login_form")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    let cleared = extract_set_cookie(&resp, "ui_state").await;
    assert_eq!(cleared.as_deref(), Some(""));
}

#[tokio::test]
async fn register_invalid_email_redirects_with_error() {
    let app = router();
    let body = "fullname=Alice+Smith&email=invalid&username=alice&password=passw0rd";
    let req = Request::post("/register")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp
        .headers()
        .get(axum::http::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.contains("error=invalid_email"));
}

#[tokio::test]
async fn ui_state_sets_cookie_and_redirects() {
    let app = router();
    let req = Request::post("/ui-state")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("to_state=login_form&return_to=%2Ffoo"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp
        .headers()
        .get(axum::http::header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(location, "/foo");
    let cookie = extract_set_cookie(&resp, "ui_state").await;
    assert_eq!(cookie.as_deref(), Some("login_form"));
}

#[tokio::test]
async fn logout_clears_ui_state_cookie() {
    let app = router();
    let req = Request::post("/logout")
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .header(axum::http::header::COOKIE, "ui_state=register_form")
        .body(Body::from(""))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    let cleared = extract_set_cookie(&resp, "ui_state").await;
    assert_eq!(cleared.as_deref(), Some(""));
}
