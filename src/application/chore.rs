use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use chrono::Days;
use crate::{domain::value::{Date, DateTime, Uuid}, AppState};
use crate::domain::chore;
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::user;
use crate::domain::authentication_session::AuthenticationSession;

#[derive(Template)]
#[template(path = "page/chore_list/chore/list.jinja")]
struct ListTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
}

pub async fn view_list(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();

    Ok(Html(ListTemplate {chore_list, chores}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/detail.jinja")]
struct DetailTemplate {
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();

    Ok(Html(DetailTemplate {chore, chore_list}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/create.jinja")]
struct CreateTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_create_form(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(CreateTemplate {chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
    points: i32,
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

    let next_due_date = if let Some(interval_days) = payload.interval_days {
        Some(Date::from(Date::now().as_ref().checked_add_days(Days::new(interval_days.into())).unwrap()))
    } else {
        None
    };

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

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/update.jinja")]
struct UpdateTemplate {
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
}

pub async fn view_update_form(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(UpdateTemplate {chore, chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
    points: i32,
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
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
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

    chore::update_next_due_date(&mut chore, &state.pool, false).await.unwrap();

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
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
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = Some(DateTime::now());

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
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
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore.date_deleted = None;

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/list_activities.jinja")]
struct ActivityListTemplate {
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    activities: Vec<chore_activity::ChoreActivity>,
    users: Vec<user::User>,
}

pub async fn view_activity_list(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let chore = match chore::get_by_id(&state.pool, &id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    let activities = chore_activity::get_all_for_chore(&state.pool, &chore.id).await.unwrap();
    let users = user::get_all(&state.pool).await.unwrap();

    Ok(Html(ActivityListTemplate {chore, chore_list, activities, users}.render().unwrap()))
}
