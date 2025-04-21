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
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use chrono::Days;
use maud::Markup;
use std::sync::Arc;

pub async fn view_list(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let (chores, deleted_chores) = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap()
        .into_iter()
        .partition(|chore| !chore.is_deleted());

    Ok(templates::page::chore_list::chore::list(chore_list, chores, deleted_chores))
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();

    Ok(templates::page::chore_list::chore::detail(chore, chore_list))
}

pub async fn view_create_form(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(templates::page::chore_list::chore::create(chore_list))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
    points: u32,
    interval_days: Option<u32>,
    description: String,
}

pub async fn create(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
    Form(payload): Form<CreatePayload>,
) -> Result<Redirect, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
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

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/chores/{}",
        chore_list.id, chore.id
    )))
}

pub async fn view_update_form(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(templates::page::chore_list::chore::update(chore, chore_list))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
    points: u32,
    interval_days: Option<u32>,
    description: String,
}

pub async fn update(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
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
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
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

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/chores/{}",
        chore_list.id, chore.id
    )))
}

pub async fn delete(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    let mut chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = Some(DateTime::now());

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/chores/{}",
        chore_list.id, chore.id
    )))
}

pub async fn restore(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    let mut chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = None;

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/chores/{}",
        chore_list.id, chore.id
    )))
}

pub async fn view_activity_list(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let users = user::get_all(&state.pool).await.unwrap();
    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    let (activities, deleted_activities): (Vec<_>, Vec<_>) = chore_activity::get_all_for_chore(&state.pool, &chore.id)
        .await
        .unwrap()
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
