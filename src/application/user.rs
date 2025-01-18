use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::user;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/user/list.jinja")]
struct ListTemplate {
    users: Vec<user::User>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Html<String> {
    let users = user::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {users}.render().unwrap())
}

#[derive(Template)]
#[template(path = "page/user/detail.jinja")]
struct DetailTemplate {
    user: user::User,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    Ok(Html(DetailTemplate {user}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/user/create.jinja")]
struct CreateTemplate();

pub async fn view_create_form(_auth_session: AuthSession) -> Html<String> {
    Html(CreateTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let user = user::User {
        id: Uuid::now_v7(),
        name: payload.name,
        date_created: chrono::offset::Utc::now(),
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
    auth_session: AuthSession,
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
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
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

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if user.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    user.date_deleted = Some(chrono::offset::Utc::now());

    user::update(&state.pool, &user).await.unwrap();

    Ok(Redirect::to(&format!("/users/{}", user.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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
