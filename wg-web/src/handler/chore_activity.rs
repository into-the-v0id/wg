use wg_core::model::chore;
use wg_core::model::chore::ChoreId;
use wg_core::model::chore_activity;
use wg_core::model::chore_activity::ChoreActivityId;
use wg_core::model::chore_list::ChoreListId;
use wg_core::model::user;
use crate::extractor::authentication::AuthSession;
use crate::extractor::model::ChoreActivity;
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
use std::sync::Arc;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities")]
pub struct ChoreActivityIndexPath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_list(
    _path: ChoreActivityIndexPath,
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (chores, users, all_activities) = tokio::try_join!(
        chore::get_all_for_chore_list(&state.pool, &chore_list.id),
        user::get_all(&state.pool),
        chore_activity::get_all_for_chore_list(&state.pool, &chore_list.id),
    ).unwrap();

    let (activities, deleted_activities): (Vec<_>, Vec<_>) = all_activities
        .into_iter()
        .partition(|activity| !activity.is_deleted());
    let activities_by_date = chore_activity::group_and_sort_by_date(activities.iter().collect(), true);

    Ok(template::page::chore_list::activity::list(
        chore_list,
        activities_by_date,
        deleted_activities,
        chores,
        users,
    ))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities/{chore_activity_id}")]
pub struct ChoreActivityDetailPath {
    pub chore_list_id: ChoreListId,
    pub chore_activity_id: ChoreActivityId,
}

pub async fn view_detail(
    _path: ChoreActivityDetailPath,
    ChoreList(chore_list): ChoreList,
    ChoreActivity(activity): ChoreActivity,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (chore, user) = tokio::try_join!(
        chore::get_by_id(&state.pool, &activity.chore_id),
        user::get_by_id(&state.pool, &activity.user_id),
    ).unwrap();

    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    let allow_edit = activity.date.as_ref() >= &min_date;

    Ok(template::page::chore_list::activity::detail(
        activity,
        chore,
        chore_list,
        user,
        auth_session,
        allow_edit,
    ))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities/create")]
pub struct ChoreActivityCreatePath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_create_form(
    _path: ChoreActivityCreatePath,
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap();
    let min_date = Date::from((chrono::Utc::now() - Days::new(2)).date_naive());
    let max_date = Date::now();
    let now = DateTime::now();

    Ok(template::page::chore_list::activity::create(
        chore_list,
        chores,
        min_date,
        max_date,
        now,
    ))
}

#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    chore_id: ChoreId,
    date: Date,
    comment: String,
}

pub async fn create(
    _path: ChoreActivityCreatePath,
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut chore = match chore::get_by_id(&state.pool, &payload.chore_id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    let max_date = chrono::Utc::now().date_naive();

    if payload.date.as_ref() < &min_date || payload.date.as_ref() > &max_date {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let activity = chore_activity::ChoreActivity {
        id: ChoreActivityId::new(),
        chore_id: chore.id,
        user_id: auth_session.user_id,
        date: payload.date,
        comment: match payload.comment.trim() {
            "" => None,
            comment => Some(comment.to_string()),
        },
        date_created: DateTime::now(),
        date_deleted: None,
    };

    chore_activity::create(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&ChoreActivityIndexPath {
        chore_list_id: chore_list.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/update")]
pub struct ChoreActivityUpdatePath {
    pub chore_list_id: ChoreListId,
    pub chore_activity_id: ChoreActivityId,
}

pub async fn view_update_form(
    _path: ChoreActivityUpdatePath,
    ChoreList(chore_list): ChoreList,
    ChoreActivity(activity): ChoreActivity,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if chore_list.is_deleted() || activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let (chore, all_chores) = tokio::try_join!(
        chore::get_by_id(&state.pool, &activity.chore_id),
        chore::get_all_for_chore_list(&state.pool, &chore_list.id),
    ).unwrap();

    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    if activity.date.as_ref() < &min_date {
        return Err(StatusCode::FORBIDDEN);
    }

    let min_date = Date::from(min_date);
    let max_date = Date::now();

    Ok(template::page::chore_list::activity::update(
        activity,
        all_chores,
        chore_list,
        min_date,
        max_date,
    ))
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    chore_id: ChoreId,
    date: Date,
    comment: String,
}

pub async fn update(
    _path: ChoreActivityUpdatePath,
    ChoreList(chore_list): ChoreList,
    ChoreActivity(mut activity): ChoreActivity,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    if activity.date.as_ref() < &min_date {
        return Err(StatusCode::FORBIDDEN);
    }

    let max_date = chrono::Utc::now().date_naive();

    if payload.date.as_ref() < &min_date || payload.date.as_ref() > &max_date {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    activity.chore_id = payload.chore_id;
    activity.date = payload.date;
    activity.comment = match payload.comment.trim() {
        "" => None,
        comment => Some(comment.to_string()),
    };

    chore_activity::update(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&ChoreActivityDetailPath {
        chore_list_id: chore_list.id,
        chore_activity_id: activity.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/delete")]
pub struct ChoreActivityDeletePath {
    pub chore_list_id: ChoreListId,
    pub chore_activity_id: ChoreActivityId,
}

pub async fn delete(
    _path: ChoreActivityDeletePath,
    ChoreList(chore_list): ChoreList,
    ChoreActivity(mut activity): ChoreActivity,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    activity.date_deleted = Some(DateTime::now());

    chore_activity::update(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&ChoreActivityDetailPath {
        chore_list_id: chore_list.id,
        chore_activity_id: activity.id,
    }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/restore")]
pub struct ChoreActivityRestorePath {
    pub chore_list_id: ChoreListId,
    pub chore_activity_id: ChoreActivityId,
}

pub async fn restore(
    _path: ChoreActivityRestorePath,
    ChoreList(chore_list): ChoreList,
    ChoreActivity(mut activity): ChoreActivity,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Redirect, StatusCode> {
    if chore_list.is_deleted() || !activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::NOT_FOUND);
    }

    activity.date_deleted = None;

    chore_activity::update(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&ChoreActivityDetailPath {
        chore_list_id: chore_list.id,
        chore_activity_id: activity.id,
    }.to_string().as_str()))
}
