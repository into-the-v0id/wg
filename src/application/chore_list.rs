use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore_list;
use crate::templates;
use crate::{
    AppState,
    domain::value::{DateTime, Uuid},
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
use std::sync::Arc;

use super::chore_activity::ChoreActivityIndexPath;

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChoreListPathData {
    chore_list_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for chore_list::ChoreList {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<ChoreListPathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let chore_list = match chore_list::get_by_id(&state.pool, &path_data.chore_list_id).await {
            Ok(chore_list) => chore_list,
            Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(chore_list)
    }
}

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

    templates::page::chore_list::list(chore_lists, deleted_chore_lists)
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/create")]
pub struct ChoreListCreatePath;

pub async fn view_create_form(
    _path: ChoreListCreatePath,
    _auth_session: AuthenticationSession,
) -> Markup {
    templates::page::chore_list::create()
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

    Redirect::to(&ChoreActivityIndexPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str())
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/update")]
pub struct ChoreListUpdatePath {
    pub chore_list_id: Uuid,
}

pub async fn view_update_form(
    _path: ChoreListUpdatePath,
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(templates::page::chore_list::update(chore_list))
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
    pub chore_list_id: Uuid,
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
    pub chore_list_id: Uuid,
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
    pub chore_list_id: Uuid,
}

pub async fn view_settings(
    _path: ChoreListSettingsPath,
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    Ok(templates::page::chore_list::settings(chore_list))
}
