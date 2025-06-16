use wg_core::model::chore;
use wg_core::model::chore::ChoreId;
use wg_core::model::chore_activity;
use wg_core::model::chore_list::ChoreListId;
use wg_core::model::user;
use crate::extractor::authentication::AuthSession;
use crate::extractor::model::Chore;
use crate::extractor::model::ChoreList;
use crate::template;
use crate::AppState;
use wg_core::value::{Date, DateTime};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use chrono::Days;
use maud::Markup;
use serde_with::serde_as;
use std::sync::Arc;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores")]
pub struct ChoreIndexPath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_list(
    _path: ChoreIndexPath,
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (chores, deleted_chores) = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap()
        .into_iter()
        .partition(|chore| !chore.is_deleted());

    Ok(template::page::chore_list::chore::list(chore_list, chores, deleted_chores))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/{chore_id}")]
pub struct ChoreDetailPath {
    pub chore_list_id: ChoreListId,
    pub chore_id: ChoreId,
}

pub async fn view_detail(
    _path: ChoreDetailPath,
    ChoreList(chore_list): ChoreList,
    Chore(chore): Chore,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(template::page::chore_list::chore::detail(chore, chore_list))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/chores/create")]
pub struct ChoreCreatePath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_create_form(
    _path: ChoreCreatePath,
    ChoreList(chore_list): ChoreList,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(template::page::chore_list::chore::create(chore_list))
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
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let next_due_date = payload.interval_days.map(|interval_days| {
        Date::from(Date::now().as_ref().clone() + Days::new(interval_days.into()))
    });

    let chore = chore::Chore {
        id: ChoreId::new(),
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
    pub chore_list_id: ChoreListId,
    pub chore_id: ChoreId,
}

pub async fn view_update_form(
    _path: ChoreUpdatePath,
    ChoreList(chore_list): ChoreList,
    Chore(chore): Chore,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() || chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(template::page::chore_list::chore::update(chore, chore_list))
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
    ChoreList(chore_list): ChoreList,
    Chore(mut chore): Chore,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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
    pub chore_list_id: ChoreListId,
    pub chore_id: ChoreId,
}

pub async fn delete(
    _path: ChoreDeletePath,
    ChoreList(chore_list): ChoreList,
    Chore(mut chore): Chore,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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
    pub chore_list_id: ChoreListId,
    pub chore_id: ChoreId,
}

pub async fn restore(
    _path: ChoreRestorePath,
    ChoreList(chore_list): ChoreList,
    Chore(mut chore): Chore,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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
    pub chore_list_id: ChoreListId,
    pub chore_id: ChoreId,
}

pub async fn view_activity_list(
    _path: ChoreActivitiesPath,
    ChoreList(chore_list): ChoreList,
    Chore(chore): Chore,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
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

    Ok(template::page::chore_list::chore::list_activities(
        chore,
        chore_list,
        activities_by_date,
        deleted_activities,
        users,
    ))
}
