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
            .route("/ui-state", post(set_ui_state))
            .route("/login", post(login))
            .route("/register", post(register))
            .route("/logout", post(logout))
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(cookies: Cookies, Form(form): Form<LoginRequest>) -> Redirect {
    match validate_login_inputs(&form.username, &form.password) {
        Ok(username) => {
            cookies.add(Cookie::new("username", username));
            // Clear any persisted UI state when successfully logging in.
            cookies.remove(Cookie::new("ui_state", ""));
        }
        Err(code) => return Redirect::to(&format!("/?error={code}")),
    }
    Redirect::to("/")
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    fullname: String,
    email: String,
    username: String,
    password: String,
}

pub async fn register(cookies: Cookies, Form(form): Form<RegisterRequest>) -> Redirect {
    match validate_register_inputs(&form.fullname, &form.email, &form.username, &form.password) {
        Ok(username) => {
            cookies.add(Cookie::new("username", username));
            // Clear any persisted UI state when successfully registering.
            cookies.remove(Cookie::new("ui_state", ""));
        }
        Err(code) => return Redirect::to(&format!("/?error={code}")),
    }
    Redirect::to("/")
}

pub async fn logout(cookies: Cookies) -> Redirect {
    cookies.remove(Cookie::new("username", ""));
    cookies.remove(Cookie::new("ui_state", ""));
    Redirect::to("/")
}

#[derive(Deserialize)]
pub struct UiStateForm {
    to_state: String,
    return_to: Option<String>,
}

pub async fn set_ui_state(cookies: Cookies, Form(form): Form<UiStateForm>) -> Redirect {
    let UiStateForm {
        to_state,
        return_to,
    } = form;
    cookies.add(Cookie::new("ui_state", to_state));
    let target = return_to.unwrap_or_else(|| "/".to_string());
    Redirect::to(&target)
}

fn validate_login_inputs(username: &str, password: &str) -> Result<String, &'static str> {
    let username = username.trim();
    let password = password.trim();
    if username.is_empty() || password.is_empty() {
        Err("missing_fields")
    } else {
        Ok(username.to_owned())
    }
}

fn validate_register_inputs(
    fullname: &str,
    email: &str,
    username: &str,
    password: &str,
) -> Result<String, &'static str> {
    let fullname = fullname.trim();
    let email = email.trim();
    let username = username.trim();
    let password = password.trim();
    if fullname.is_empty() || email.is_empty() || username.is_empty() || password.is_empty() {
        return Err("missing_fields");
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("invalid_email");
    }
    if password.len() < 8 {
        return Err("weak_password");
    }
    Ok(username.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{validate_login_inputs, validate_register_inputs};

    #[test]
    fn login_rejects_missing() {
        assert_eq!(validate_login_inputs("", "password"), Err("missing_fields"));
        assert_eq!(validate_login_inputs("user", ""), Err("missing_fields"));
    }

    #[test]
    fn login_accepts_trimmed() {
        assert_eq!(
            validate_login_inputs("  alice  ", "  passw0rd  ").unwrap(),
            "alice"
        );
    }

    #[test]
    fn register_rejects_missing() {
        assert_eq!(
            validate_register_inputs("", "a@b.com", "user", "password"),
            Err("missing_fields")
        );
        assert_eq!(
            validate_register_inputs("name", "", "user", "password"),
            Err("missing_fields")
        );
    }

    #[test]
    fn register_rejects_invalid_email() {
        assert_eq!(
            validate_register_inputs("name", "not-an-email", "user", "password"),
            Err("invalid_email")
        );
    }

    #[test]
    fn register_rejects_weak_password() {
        assert_eq!(
            validate_register_inputs("name", "a@b.com", "user", "short"),
            Err("weak_password")
        );
    }

    #[test]
    fn register_accepts_valid() {
        assert_eq!(
            validate_register_inputs(" Name ", " user@example.com ", " user ", " passw0rd ")
                .unwrap(),
            "user"
        );
    }
}
