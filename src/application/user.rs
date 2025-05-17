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
use axum_extra::routing::TypedPath;
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

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users")]
pub struct UserIndexPath;

pub async fn view_list(
    _path: UserIndexPath,
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

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}")]
pub struct UserDetailPath {
    pub user_id: Uuid,
}

pub async fn view_detail(
    _path: UserDetailPath,
    user: user::User,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    Ok(templates::page::user::detail(user))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/create")]
pub struct UserCreatePath;

pub async fn view_create_form(
    _path: UserCreatePath,
    _auth_session: AuthenticationSession,
) -> Markup {
    templates::page::user::create()
}

#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    name: String,
    handle: String,
    password: SecretString,
}

pub async fn create(
    _path: UserCreatePath,
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

    Redirect::to(UserDetailPath {
        user_id: user.id,
    }.to_string().as_str())
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}/update")]
pub struct UserUpdatePath {
    pub user_id: Uuid,
}

pub async fn view_update_form(
    _path: UserUpdatePath,
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
    _path: UserUpdatePath,
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

    Ok(Redirect::to(UserDetailPath {
        user_id: user.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}/delete")]
pub struct UserDeletePath {
    pub user_id: Uuid,
}

pub async fn delete(
    _path: UserDeletePath,
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
    let auth_session_deletions = auth_sessions
        .iter()
        .map(|auth_session| authentication_session::delete(&state.pool, auth_session));
    futures::future::join_all(auth_session_deletions).await;

    Ok(Redirect::to(UserDetailPath {
        user_id: user.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}/restore")]
pub struct UserRestorePath {
    pub user_id: Uuid,
}

pub async fn restore(
    _path: UserRestorePath,
    mut user: user::User,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if !user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = None;

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(UserDetailPath {
        user_id: user.id,
    }.to_string().as_str()))
}
