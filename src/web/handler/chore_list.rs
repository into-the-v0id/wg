use crate::model::authentication_session::AuthenticationSession;
use crate::model::chore_list::{self, ChoreListId};
use crate::web::template;
use crate::{
    AppState,
    value::{DateTime},
};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use maud::Markup;
use std::sync::Arc;

use super::chore_activity::ChoreActivityIndexPath;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists")]
pub struct ChoreListIndexPath;

pub async fn view_list(
    _path: ChoreListIndexPath,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Markup {
    let (chore_lists, deleted_chore_lists) = chore_list::get_all(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .partition(|chore_list| !chore_list.is_deleted());

    template::page::chore_list::list(chore_lists, deleted_chore_lists)
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/create")]
pub struct ChoreListCreatePath;

pub async fn view_create_form(
    _path: ChoreListCreatePath,
    _auth_session: AuthenticationSession,
) -> Markup {
    template::page::chore_list::create()
}

#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    name: String,
    description: String,
    score_reset_interval: chore_list::ScoreResetInterval,
}

pub async fn create(
    _path: ChoreListCreatePath,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let chore_list = chore_list::ChoreList {
        id: ChoreListId::new(),
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

    Redirect::to(&ChoreActivityIndexPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str())
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/update")]
pub struct ChoreListUpdatePath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_update_form(
    _path: ChoreListUpdatePath,
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(template::page::chore_list::update(chore_list))
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    name: String,
    description: String,
    score_reset_interval: chore_list::ScoreResetInterval,
}

pub async fn update(
    _path: ChoreListUpdatePath,
    mut chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
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

    Ok(Redirect::to(ChoreListSettingsPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/delete")]
pub struct ChoreListDeletePath {
    pub chore_list_id: ChoreListId,
}

pub async fn delete(
    _path: ChoreListDeletePath,
    mut chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = Some(DateTime::now());

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(ChoreListSettingsPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/restore")]
pub struct ChoreListRestorePath {
    pub chore_list_id: ChoreListId,
}

pub async fn restore(
    _path: ChoreListRestorePath,
    mut chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if !chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = None;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(ChoreListSettingsPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/settings")]
pub struct ChoreListSettingsPath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_settings(
    _path: ChoreListSettingsPath,
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    Ok(template::page::chore_list::settings(chore_list))
}
