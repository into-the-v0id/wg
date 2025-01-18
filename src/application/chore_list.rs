use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::chore_list;
use crate::domain::chore;

#[derive(Template)]
#[template(path = "page/chore_list/list.jinja")]
struct ListTemplate {
    chore_lists: Vec<chore_list::ChoreList>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let chore_lists = chore_list::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {chore_lists}.render().unwrap())
}

#[derive(Template)]
#[template(path = "page/chore_list/detail.jinja")]
struct DetailTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    Ok(Html(DetailTemplate {chore_list}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/create.jinja")]
struct CreateTemplate();

pub async fn view_create_form() -> Html<String> {
    Html(CreateTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let chore_list = chore_list::ChoreList {
        id: Uuid::now_v7(),
        name: payload.name,
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore_list::create(&state.pool, &chore_list).await.unwrap();

    Redirect::to(&format!("/chore-lists/{}", chore_list.id))
}

#[derive(Template)]
#[template(path = "page/chore_list/update.jinja")]
struct UpdateTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(UpdateTemplate {chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.name = payload.name;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = Some(chrono::offset::Utc::now());

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to("/chore-lists"))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if !chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = None;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to("/chore-lists"))
}

#[derive(Template)]
#[template(path = "page/chore_list/list_chores.jinja")]
struct ChoreListTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
}

pub async fn view_chore_list(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();

    Ok(Html(ChoreListTemplate {chore_list, chores}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/create_chore.jinja")]
struct CreateChoreTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_create_chore_form(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(CreateChoreTemplate {chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreateChorePayload {
    name: String,
    points: i32,
}

pub async fn create_chore(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreateChorePayload>,
) -> Result<Redirect, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::Chore {
        id: Uuid::now_v7(),
        chore_list_id: chore_list.id,
        name: payload.name,
        points: payload.points,
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore::create(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chores/{}", chore.id)))
}
