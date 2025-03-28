use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use strum::IntoEnumIterator;
use crate::{domain::value::{DateTime, Uuid}, AppState};
use crate::domain::chore_list;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/chore_list/list.jinja")]
struct ListTemplate {
    chore_lists: Vec<chore_list::ChoreList>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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
    _auth_session: AuthSession,
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
struct CreateTemplate {
    score_reset_intervals: Vec<chore_list::ScoreResetInterval>,
}

pub async fn view_create_form(_auth_session: AuthSession) -> Html<String> {
    let score_reset_intervals = chore_list::ScoreResetInterval::iter().collect();

    Html(CreateTemplate{score_reset_intervals}.render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
    description: String,
    score_reset_interval: chore_list::ScoreResetInterval,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let chore_list = chore_list::ChoreList {
        id: Uuid::new(),
        name: payload.name,
        description: match payload.description.trim() {
            "" => None,
            description => Some(description.to_string()),
        },
        score_reset_interval: payload.score_reset_interval,
        date_created: DateTime::now(),
        date_deleted: None,
    };

    chore_list::create(&state.pool, &chore_list).await.unwrap();

    Redirect::to(&format!("/chore-lists/{}", chore_list.id))
}

#[derive(Template)]
#[template(path = "page/chore_list/update.jinja")]
struct UpdateTemplate {
    chore_list: chore_list::ChoreList,
    score_reset_intervals: Vec<chore_list::ScoreResetInterval>,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let score_reset_intervals = chore_list::ScoreResetInterval::iter().collect();

    Ok(Html(UpdateTemplate {chore_list, score_reset_intervals}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
    description: String,
    score_reset_interval: chore_list::ScoreResetInterval,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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
    chore_list.description = match payload.description.trim() {
        "" => None,
        description => Some(description.to_string()),
    };
    chore_list.score_reset_interval = payload.score_reset_interval;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = Some(DateTime::now());

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}
