use std::sync::Arc;

use askama::Template;
use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2};
use axum::{extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, response::{Html, Redirect}, Form, RequestPartsExt};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use uuid::Uuid;
use crate::AppState;
use crate::domain::user;

const COOKIE_NAME: &str = "authentication";

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub id: Uuid,
    pub auth_token: String,
    pub user_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AuthSession
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();

        let auth_token = match cookie_jar.get(COOKIE_NAME) {
            Some(auth_token) => auth_token.value(),
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        let auth_sessions = state.auth_sessions.lock().await;
        let auth_session = match auth_sessions.iter().find(|s| s.auth_token == auth_token) {
            Some(auth_session) => auth_session,
            None => return Err(StatusCode::UNAUTHORIZED)
        };

        Ok(auth_session.clone())
    }
}

#[derive(Template)]
#[template(path = "page/authentication/login.jinja")]
struct LoginTemplate();

pub async fn view_login_form() -> Html<String> {
    Html(LoginTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct LoginPayload {
    handle: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    cookie_jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let user = match user::get_by_handle(&state.pool, &payload.handle).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let is_valid_password = Argon2::default()
        .verify_password(
            payload.password.as_bytes(),
            &PasswordHash::new(&user.password_hash).unwrap(),
        )
        .is_ok();
    if ! is_valid_password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut auth_token_buf = [0u8; 64];
    getrandom::getrandom(&mut auth_token_buf).unwrap();
    let auth_token = const_hex::encode(auth_token_buf);

    let auth_session = AuthSession {
        id: Uuid::now_v7(),
        auth_token,
        user_id: user.id,
    };

    let cookie =  Cookie::build((COOKIE_NAME, auth_session.auth_token.clone()))
        .secure(true)
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();
    let cookie_jar = cookie_jar.add(cookie);

    let mut auth_sessions = state.auth_sessions.lock().await;
    auth_sessions.push(auth_session);

    Ok((cookie_jar, Redirect::to("/")))
}
