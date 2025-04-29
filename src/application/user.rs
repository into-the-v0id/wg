use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::user;
use crate::templates;
use crate::{
    AppState,
    domain::{
        authentication_session,
        value::{DateTime, PasswordHash, Uuid},
    },
};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use maud::Markup;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct UserPathData {
    user_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for user::User {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<UserPathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let user = match user::get_by_id(&state.pool, &path_data.user_id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(user)
    }
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Markup {
    let (users, deleted_users) = user::get_all(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .partition(|user| !user.is_deleted());

    templates::page::user::list(users, deleted_users)
}

pub async fn view_detail(
    user: user::User,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    Ok(templates::page::user::detail(user))
}

pub async fn view_create_form(_auth_session: AuthenticationSession) -> Markup {
    templates::page::user::create()
}

#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    name: String,
    handle: String,
    password: SecretString,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let user = user::User {
        id: Uuid::new(),
        name: payload.name,
        handle: payload.handle,
        password_hash: PasswordHash::from_plain_password(payload.password),
        date_created: DateTime::now(),
        date_deleted: None,
    };

    user::create(&state.pool, &user).await.unwrap();

    Redirect::to(&format!("/users/{}", user.id))
}

pub async fn view_update_form(
    user: user::User,
    auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if user.id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(templates::page::user::update(user))
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    name: String,
    handle: String,
    password: SecretString,
}

pub async fn update(
    mut user: user::User,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if user.id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    user.name = payload.name;
    user.handle = payload.handle;

    if !payload.password.expose_secret().trim().is_empty() {
        user.password_hash = PasswordHash::from_plain_password(payload.password);
    }

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn delete(
    mut user: user::User,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = Some(DateTime::now());

    user::update(&state.pool, &user).await.unwrap();

    // Remove auth sessions for that user
    let auth_sessions = authentication_session::get_all_for_user(&state.pool, &user.id)
        .await
        .unwrap();
    for auth_session in auth_sessions.iter() {
        authentication_session::delete(&state.pool, auth_session)
            .await
            .unwrap();
    }

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn restore(
    mut user: user::User,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if !user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = None;

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}
