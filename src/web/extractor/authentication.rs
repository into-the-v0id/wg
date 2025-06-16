use std::sync::Arc;
use crate::{
    web::AppState,
    model::{authentication_session::{self, AuthenticationSession}},
};
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;

pub const COOKIE_NAME: &str = "authentication";

impl FromRequestParts<Arc<AppState>> for AuthenticationSession {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_session) = parts.extensions.get::<AuthenticationSession>() {
            if auth_session.is_expired() {
                return Err(StatusCode::UNAUTHORIZED);
            }

            return Ok(auth_session.clone());
        };

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

        parts.extensions.insert::<AuthenticationSession>(auth_session.clone());

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
