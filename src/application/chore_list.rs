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
use maud::Markup;
use std::sync::Arc;

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

pub async fn view_list(
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

pub async fn view_create_form(_auth_session: AuthenticationSession) -> Markup {
    templates::page::chore_list::create()
}

#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    name: String,
    description: String,
    score_reset_interval: chore_list::ScoreResetInterval,
}

pub async fn create(
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

    Redirect::to(&format!("/chore-lists/{}/activities", chore_list.id))
}

pub async fn view_update_form(
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

    Ok(Redirect::to(&format!("/chore-lists/{}/settings", chore_list.id)))
}

pub async fn delete(
    mut chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = Some(DateTime::now());

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/settings", chore_list.id)))
}

pub async fn restore(
    mut chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if !chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = None;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/settings", chore_list.id)))
}

pub async fn view_settings(
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    Ok(templates::page::chore_list::settings(chore_list))
}
