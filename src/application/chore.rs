use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::chore;
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::user;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/chore_list/chore/list.jinja")]
struct ListTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
}

pub async fn view_list(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
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
    _auth_session: AuthSession,
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
    _auth_session: AuthSession,
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
    description: String,
}

pub async fn create(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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

    let chore = chore::Chore {
        id: Uuid::now_v7(),
        chore_list_id: chore_list.id,
        name: payload.name,
        points: payload.points,
        description: match payload.description.trim() {
            "" => None,
            description => Some(description.to_string()),
        },
        date_created: chrono::offset::Utc::now(),
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
    _auth_session: AuthSession,
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
    description: String,
}

pub async fn update(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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
    chore.description = match payload.description.trim() {
        "" => None,
        description => Some(description.to_string()),
    };

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
}

pub async fn delete(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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

    chore.date_deleted = Some(chrono::offset::Utc::now());

    chore::update(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
}

pub async fn restore(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
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
    _auth_session: AuthSession,
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
