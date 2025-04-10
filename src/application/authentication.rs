use std::sync::Arc;

use askama::Template;
use axum::{extract::{FromRequestParts, OptionalFromRequestParts, State}, http::{request::Parts, StatusCode}, response::{Html, IntoResponse, Redirect}, Form, RequestPartsExt};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use secrecy::SecretString;
use crate::{domain::value::Uuid, AppState};
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

impl OptionalFromRequestParts<Arc<AppState>> for AuthSession
{
    type Rejection = <AuthSession as FromRequestParts<Arc<AppState>>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Option<Self>, Self::Rejection> {
        match <AuthSession as FromRequestParts<Arc<AppState>>>::from_request_parts(parts, state).await {
            Ok(auth_session) => Ok(Some(auth_session)),
            Err(StatusCode::UNAUTHORIZED) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[derive(Template)]
#[template(path = "page/authentication/login.jinja")]
struct LoginTemplate();

pub async fn view_login_form(auth_session: Option<AuthSession>) -> impl IntoResponse {
    if auth_session.is_some() {
        return Redirect::to("/").into_response();
    }

    Html(LoginTemplate().render().unwrap()).into_response()
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct LoginPayload {
    handle: String,
    password: SecretString,
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

    let is_matching_password = user.password_hash.verify(payload.password);
    if ! is_matching_password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut auth_token_buf = [0u8; 64];
    getrandom::getrandom(&mut auth_token_buf).unwrap();
    let auth_token = const_hex::encode(auth_token_buf);

    let auth_session = AuthSession {
        id: Uuid::new(),
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

pub async fn logout(
    auth_session: Option<AuthSession>,
    State(state): State<Arc<AppState>>,
    mut cookie_jar: CookieJar,
) -> (CookieJar, Redirect) {
    if let Some(auth_session) = auth_session {
        let mut auth_sessions = state.auth_sessions.lock().await;
        if let Some(auth_session_position) = auth_sessions.iter().position(|item| item.id == auth_session.id) {
            auth_sessions.remove(auth_session_position);
        }
    }

    if cookie_jar.get(COOKIE_NAME).is_some() {
        cookie_jar = cookie_jar.remove(COOKIE_NAME);
    }

    (cookie_jar, Redirect::to("/login"))
}
