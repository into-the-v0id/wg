use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::chore;
use crate::domain::chore_list;

#[derive(Template)]
#[template(path = "page/chore/detail.jinja")]
struct DetailTemplate {
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();

    Ok(Html(DetailTemplate {chore, chore_list}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore/update.jinja")]
struct UpdateTemplate {
    chore: chore::Chore,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(UpdateTemplate {chore}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
    points: i32,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.name = payload.name;
    chore.points = payload.points;

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chores/{}", chore.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, StatusCode> {
    let mut chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = Some(chrono::offset::Utc::now());

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, StatusCode> {
    let mut chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = None;

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to("/chores"))
}
