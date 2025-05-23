use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore;
use crate::domain::chore_activity;
use crate::domain::chore_list;
use crate::domain::user;
use crate::templates;
use crate::{
    AppState,
    domain::value::{Date, DateTime, Uuid},
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
use chrono::Days;
use maud::Markup;
use serde_with::serde_as;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChorePathData {
    chore_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for chore::Chore {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<ChorePathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let chore = match chore::get_by_id(&state.pool, &path_data.chore_id).await {
            Ok(chore) => chore,
            Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(chore)
    }
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores")]
pub struct ChoreIndexPath {
    pub chore_list_id: Uuid,
}

pub async fn view_list(
    _path: ChoreIndexPath,
    chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let (chores, deleted_chores) = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap()
        .into_iter()
        .partition(|chore| !chore.is_deleted());

    Ok(templates::page::chore_list::chore::list(chore_list, chores, deleted_chores))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}")]
pub struct ChoreDetailPath {
    pub chore_list_id: Uuid,
    pub chore_id: Uuid,
}

pub async fn view_detail(
    _path: ChoreDetailPath,
    chore_list: chore_list::ChoreList,
    chore: chore::Chore,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(templates::page::chore_list::chore::detail(chore, chore_list))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/create")]
pub struct ChoreCreatePath {
    pub chore_list_id: Uuid,
}

pub async fn view_create_form(
    _path: ChoreCreatePath,
    chore_list: chore_list::ChoreList,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(templates::page::chore_list::chore::create(chore_list))
}

#[serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    name: String,
    points: u32,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    interval_days: Option<u32>,
    description: String,
}

pub async fn create(
    _path: ChoreCreatePath,
    chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<CreatePayload>,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let next_due_date = payload.interval_days.map(|interval_days| {
        Date::from(Date::now().as_ref().clone() + Days::new(interval_days.into()))
    });

    let chore = chore::Chore {
        id: Uuid::new(),
        chore_list_id: chore_list.id,
        name: payload.name,
        points: payload.points,
        interval_days: payload.interval_days,
        next_due_date,
        description: match payload.description.trim() {
            "" => None,
            description => Some(description.to_string()),
        },
        date_created: DateTime::now(),
        date_deleted: None,
    };

    chore::create(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(ChoreDetailPath {
        chore_list_id: chore_list.id,
        chore_id: chore.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}/update")]
pub struct ChoreUpdatePath {
    pub chore_list_id: Uuid,
    pub chore_id: Uuid,
}

pub async fn view_update_form(
    _path: ChoreUpdatePath,
    chore_list: chore_list::ChoreList,
    chore: chore::Chore,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() || chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(templates::page::chore_list::chore::update(chore, chore_list))
}

#[serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    name: String,
    points: u32,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    interval_days: Option<u32>,
    description: String,
}

pub async fn update(
    _path: ChoreUpdatePath,
    chore_list: chore_list::ChoreList,
    mut chore: chore::Chore,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    chore.name = payload.name;
    chore.points = payload.points;
    chore.interval_days = payload.interval_days;
    chore.description = match payload.description.trim() {
        "" => None,
        description => Some(description.to_string()),
    };

    chore::update_next_due_date(&mut chore, &state.pool, false)
        .await
        .unwrap();

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(ChoreDetailPath {
        chore_list_id: chore_list.id,
        chore_id: chore.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}/delete")]
pub struct ChoreDeletePath {
    pub chore_list_id: Uuid,
    pub chore_id: Uuid,
}

pub async fn delete(
    _path: ChoreDeletePath,
    chore_list: chore_list::ChoreList,
    mut chore: chore::Chore,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    chore.date_deleted = Some(DateTime::now());

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(ChoreDetailPath {
        chore_list_id: chore_list.id,
        chore_id: chore.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}/restore")]
pub struct ChoreRestorePath {
    pub chore_list_id: Uuid,
    pub chore_id: Uuid,
}

pub async fn restore(
    _path: ChoreRestorePath,
    chore_list: chore_list::ChoreList,
    mut chore: chore::Chore,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || !chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    chore.date_deleted = None;

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(ChoreDetailPath {
        chore_list_id: chore_list.id,
        chore_id: chore.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}/activities")]
pub struct ChoreActivitiesPath {
    pub chore_list_id: Uuid,
    pub chore_id: Uuid,
}

pub async fn view_activity_list(
    _path: ChoreActivitiesPath,
    chore_list: chore_list::ChoreList,
    chore: chore::Chore,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let (users, all_activities) = tokio::try_join!(
        user::get_all(&state.pool),
        chore_activity::get_all_for_chore(&state.pool, &chore.id),
    ).unwrap();

    let (activities, deleted_activities): (Vec<_>, Vec<_>) = all_activities
        .into_iter()
        .partition(|activity| !activity.is_deleted());
    let activities_by_date = chore_activity::group_and_sort_by_date(activities.iter().collect(), true);

    Ok(templates::page::chore_list::chore::list_activities(
        chore,
        chore_list,
        activities_by_date,
        deleted_activities,
        users,
    ))
}
