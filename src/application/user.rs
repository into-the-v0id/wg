use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use secrecy::{ExposeSecret, SecretString};
use crate::{domain::{authentication_session, value::{DateTime, PasswordHash, Uuid}}, AppState};
use crate::domain::user;
use crate::domain::authentication_session::AuthenticationSession;

#[derive(Template)]
#[template(path = "page/user/list.jinja")]
struct ListTemplate {
    users: Vec<user::User>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Html<String> {
    let users = user::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {users}.render().unwrap())
}

#[derive(Template)]
#[template(path = "page/user/detail.jinja")]
struct DetailTemplate {
    user: user::User,
    auth_session: AuthenticationSession,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    Ok(Html(DetailTemplate {user, auth_session}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/user/create.jinja")]
struct CreateTemplate();

pub async fn view_create_form(_auth_session: AuthenticationSession) -> Html<String> {
    Html(CreateTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
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

#[derive(Template)]
#[template(path = "page/user/update.jinja")]
struct UpdateTemplate {
    user: user::User,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if user.id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(UpdateTemplate {user}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
    handle: String,
    password: SecretString,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if user.id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    user.name = payload.name;
    user.handle = payload.handle;

    if ! payload.password.expose_secret().trim().is_empty() {
        user.password_hash = PasswordHash::from_plain_password(payload.password);
    }

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    let mut user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = Some(DateTime::now());

    user::update(&state.pool, &user).await.unwrap();

    // Remove auth sessions for that user
    let auth_sessions = authentication_session::get_all_for_user(&state.pool, &user.id).await.unwrap();
    for auth_session in auth_sessions.iter() {
        authentication_session::delete(&state.pool, auth_session).await.unwrap();
    }

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    let mut user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = None;

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}
