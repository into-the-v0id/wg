use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::chore_list;
use crate::domain::user;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/chore_activity/detail.jinja")]
struct DetailTemplate {
    activity: chore_activity::ChoreActivity,
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    user: user::User,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    let user = user::get_by_id(&state.pool, &activity.user_id).await.unwrap();

    Ok(Html(DetailTemplate {activity, chore, chore_list, user}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_activity/update.jinja")]
struct UpdateTemplate {
    activity: chore_activity::ChoreActivity,
    chores: Vec<chore::Chore>,
    users: Vec<user::User>,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let users = user::get_all(&state.pool).await.unwrap();

    Ok(Html(UpdateTemplate {activity, chores, users}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    chore_id: Uuid,
    user_id: Uuid,
    date: chrono::NaiveDate,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut chore_activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &chore_activity.chore_id).await.unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_activity.chore_id = payload.chore_id;
    chore_activity.user_id = payload.user_id;
    chore_activity.date = payload.date;

    chore_activity::update(&state.pool, &chore_activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-activities/{}", chore_activity.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut chore_activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &chore_activity.chore_id).await.unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_activity.date_deleted = Some(chrono::offset::Utc::now());

    chore_activity::update(&state.pool, &chore_activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities", chore_list.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut chore_activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !chore_activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &chore_activity.chore_id).await.unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_activity.date_deleted = None;

    chore_activity::update(&state.pool, &chore_activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities", chore_list.id)))
}
