use std::sync::Arc;

use wg_core::model::authentication_session::AuthenticationSessionId;
use crate::extractor::authentication::{AuthSession, COOKIE_NAME};
use crate::template;
use wg_core::model::user;
use wg_core::value::DateTime;
use crate::AppState;
use wg_core::model::{authentication_session::{self, AuthenticationSession}};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use axum_extra::routing::TypedPath;
use chrono::Days;
use secrecy::SecretString;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/login")]
pub struct LoginPath;

pub async fn view_login_form(
    _path: LoginPath,
    auth_session: Option<AuthSession>,
) -> impl IntoResponse {
    if auth_session.is_some() {
        return Redirect::to("/").into_response();
    }

    template::page::authentication::login().into_response()
}

#[derive(serde::Deserialize, Debug)]
pub struct LoginPayload {
    email: String,
    password: SecretString,
}

pub async fn login(
    _path: LoginPath,
    State(state): State<Arc<AppState>>,
    cookie_jar: CookieJar,
    Form(payload): Form<LoginPayload>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let user = match user::get_by_email(&state.pool, &payload.email).await {
        Ok(user) => user,
        Err(wg_core::db::sqlx::Error::RowNotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let is_matching_password = user.password_hash.verify(payload.password);
    if !is_matching_password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut token_buf = [0u8; 64];
    getrandom::getrandom(&mut token_buf).unwrap();
    let token = const_hex::encode(token_buf);

    let auth_session = AuthenticationSession {
        id: AuthenticationSessionId::new(),
        token,
        user_id: user.id,
        date_expires: DateTime::from(DateTime::now().as_ref().clone() + Days::new(30)),
        date_created: DateTime::now(),
    };

    authentication_session::create(&state.pool, &auth_session)
        .await
        .unwrap();

    authentication_session::delete_all_expired(&state.pool)
        .await
        .unwrap();

    let cookie = Cookie::build((COOKIE_NAME, auth_session.token.clone()))
        .secure(true)
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .expires(
            time::OffsetDateTime::from_unix_timestamp(
                auth_session.date_expires.as_ref().timestamp(),
            )
            .unwrap(),
        )
        .build();
    let cookie_jar = cookie_jar.add(cookie);

    Ok((cookie_jar, Redirect::to("/")))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/logout")]
pub struct LogoutPath;

pub async fn logout(
    _path: LogoutPath,
    auth_session: Option<AuthSession>,
    State(state): State<Arc<AppState>>,
    mut cookie_jar: CookieJar,
) -> (CookieJar, Redirect) {
    if let Some(AuthSession(auth_session)) = auth_session {
        authentication_session::delete(&state.pool, &auth_session)
            .await
            .unwrap();
    }

    if cookie_jar.get(COOKIE_NAME).is_some() {
        cookie_jar = cookie_jar.remove(COOKIE_NAME);
    }

    (cookie_jar, Redirect::to(LoginPath.to_string().as_str()))
}
