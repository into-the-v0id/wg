use wg_core::model::user::{self, UserId};
use crate::extractor::authentication::AuthSession;
use crate::extractor::model::User;
use crate::template;
use wg_core::value::{DateTime, PasswordHash};
use crate::AppState;
use wg_core::model::authentication_session;
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use maud::Markup;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use super::settings::SettingsIndexPath;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users")]
pub struct UserIndexPath;

pub async fn view_list(
    _path: UserIndexPath,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Markup {
    let (users, deleted_users) = user::get_all(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .partition(|user| !user.is_deleted());

    template::page::user::list(users, deleted_users)
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}")]
pub struct UserDetailPath {
    pub user_id: UserId,
}

pub async fn view_detail(
    _path: UserDetailPath,
    User(user): User,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    Ok(template::page::user::detail(user))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/create")]
pub struct UserCreatePath;

pub async fn view_create_form(
    _path: UserCreatePath,
    AuthSession(_auth_session): AuthSession,
) -> Markup {
    template::page::user::create()
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
    AuthSession(_auth_session): AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let user = user::User {
        id: UserId::new(),
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
    pub user_id: UserId,
}

pub async fn view_update_form(
    _path: UserUpdatePath,
    User(user): User,
    AuthSession(auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if user.id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(template::page::user::update(user))
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    name: String,
    handle: String,
    password: SecretString,
}

pub async fn update(
    _path: UserUpdatePath,
    User(mut user): User,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
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

    Ok(Redirect::to(SettingsIndexPath.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/users/{user_id}/delete")]
pub struct UserDeletePath {
    pub user_id: UserId,
}

pub async fn delete(
    _path: UserDeletePath,
    User(mut user): User,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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
    pub user_id: UserId,
}

pub async fn restore(
    _path: UserRestorePath,
    User(mut user): User,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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
