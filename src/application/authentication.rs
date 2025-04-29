use std::sync::Arc;

use crate::templates;
use crate::domain::user;
use crate::{
    AppState,
    domain::{
        authentication_session::{self, AuthenticationSession},
        value::{DateTime, Uuid},
    },
};
use axum::{
    Form, RequestPartsExt,
    extract::{FromRequestParts, OptionalFromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use chrono::Days;
use secrecy::SecretString;

const COOKIE_NAME: &str = "authentication";

impl FromRequestParts<Arc<AppState>> for AuthenticationSession {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();

        let auth_token = match cookie_jar.get(COOKIE_NAME) {
            Some(auth_token) => auth_token.value(),
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        let auth_session = match authentication_session::get_by_token(&state.pool, auth_token).await
        {
            Ok(auth_session) => auth_session,
            Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNAUTHORIZED),
            Err(err) => panic!("{}", err),
        };
        if auth_session.is_expired() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(auth_session)
    }
}

impl OptionalFromRequestParts<Arc<AppState>> for AuthenticationSession {
    type Rejection = <AuthenticationSession as FromRequestParts<Arc<AppState>>>::Rejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <AuthenticationSession as FromRequestParts<Arc<AppState>>>::from_request_parts(
            parts, state,
        )
        .await
        {
            Ok(auth_session) => Ok(Some(auth_session)),
            Err(StatusCode::UNAUTHORIZED) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

pub async fn view_login_form(auth_session: Option<AuthenticationSession>) -> impl IntoResponse {
    if auth_session.is_some() {
        return Redirect::to("/").into_response();
    }

    templates::page::authentication::login().into_response()
}

#[derive(serde::Deserialize, Debug)]
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
    if !is_matching_password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut token_buf = [0u8; 64];
    getrandom::getrandom(&mut token_buf).unwrap();
    let token = const_hex::encode(token_buf);

    let auth_session = AuthenticationSession {
        id: Uuid::new(),
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

pub async fn logout(
    auth_session: Option<AuthenticationSession>,
    State(state): State<Arc<AppState>>,
    mut cookie_jar: CookieJar,
) -> (CookieJar, Redirect) {
    if let Some(auth_session) = auth_session {
        authentication_session::delete(&state.pool, &auth_session)
            .await
            .unwrap();
    }

    if cookie_jar.get(COOKIE_NAME).is_some() {
        cookie_jar = cookie_jar.remove(COOKIE_NAME);
    }

    (cookie_jar, Redirect::to("/login"))
}
