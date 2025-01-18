use std::sync::Arc;

use askama::Template;
use axum::{extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, response::{Html, Redirect}, Form, RequestPartsExt};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use uuid::Uuid;
use crate::AppState;
use crate::domain::user;

const COOKIE_NAME: &str = "authentication";

#[derive(Debug)]
pub struct AuthSession {
    id: Uuid,
    auth_token: String,
    user_id: Uuid,
}

#[derive(Debug)]
pub struct AuthSessionExtractor {
    auth_session_id: Uuid,
    user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthSessionExtractor
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();
        // let cookies = CookieJar::from_request_parts(parts, ctx).await.unwrap();

        let auth_token = match cookie_jar.get(COOKIE_NAME) {
            Some(auth_token) => auth_token.value(),
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        let auth_sessions = state.auth_sessions.lock().await;
        let auth_session = match auth_sessions.iter().find(|s| s.auth_token == auth_token) {
            Some(auth_session) => auth_session,
            None => return Err(StatusCode::UNAUTHORIZED)
        };

        Ok(AuthSessionExtractor {auth_session_id: auth_session.id, user_id: auth_session.user_id})
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
    name: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    cookie_jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let user = match user::get_by_name(&state.pool, &payload.name).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(err) => panic!("{}", err),
    };

    let auth_session = AuthSession {
        id: Uuid::now_v7(),
        auth_token: Uuid::new_v4().to_string(), // TODO: make more secure
        user_id: user.id,
    };

    let cookie_jar = cookie_jar.add(Cookie::new(COOKIE_NAME, auth_session.auth_token.clone()));

    let mut auth_sessions = state.auth_sessions.lock().await;
    auth_sessions.push(auth_session);

    Ok((cookie_jar, Redirect::to("/")))
}
