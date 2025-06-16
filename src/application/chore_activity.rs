use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore;
use crate::domain::chore::ChoreId;
use crate::domain::chore_activity;
use crate::domain::chore_activity::ChoreActivityId;
use crate::domain::chore_list;
use crate::domain::chore_list::ChoreListId;
use crate::domain::user;
use crate::templates;
use crate::{
    AppState,
    domain::value::{Date, DateTime},
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
use std::sync::Arc;

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChoreActivityPathData {
    chore_activity_id: ChoreActivityId,
}

impl FromRequestParts<Arc<AppState>> for chore_activity::ChoreActivity {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<ChoreActivityPathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let activity = match chore_activity::get_by_id(&state.pool, &path_data.chore_activity_id).await {
            Ok(activity) => activity,
            Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(activity)
    }
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/activities")]
pub struct ChoreActivityIndexPath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_list(
    _path: ChoreActivityIndexPath,
    chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
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

    Ok(templates::page::chore_list::activity::list(
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
    chore_list: chore_list::ChoreList,
    activity: chore_activity::ChoreActivity,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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

    Ok(templates::page::chore_list::activity::detail(
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
    chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
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

    Ok(templates::page::chore_list::activity::create(
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
    chore_list: chore_list::ChoreList,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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
    chore_list: chore_list::ChoreList,
    activity: chore_activity::ChoreActivity,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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

    Ok(templates::page::chore_list::activity::update(
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
    chore_list: chore_list::ChoreList,
    mut activity: chore_activity::ChoreActivity,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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
    chore_list: chore_list::ChoreList,
    mut activity: chore_activity::ChoreActivity,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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
    chore_list: chore_list::ChoreList,
    mut activity: chore_activity::ChoreActivity,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
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
