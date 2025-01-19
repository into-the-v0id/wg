use std::{collections::HashMap, sync::Arc};
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::{domain::user, AppState};
use crate::domain::chore_list;
use crate::domain::chore;
use crate::domain::chore_activity;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/chore_list/list.jinja")]
struct ListTemplate {
    chore_lists: Vec<chore_list::ChoreList>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Html<String> {
    let chore_lists = chore_list::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {chore_lists}.render().unwrap())
}

#[derive(Template)]
#[template(path = "page/chore_list/detail.jinja")]
struct DetailTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    Ok(Html(DetailTemplate {chore_list}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/create.jinja")]
struct CreateTemplate();

pub async fn view_create_form(_auth_session: AuthSession) -> Html<String> {
    Html(CreateTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    name: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let chore_list = chore_list::ChoreList {
        id: Uuid::now_v7(),
        name: payload.name,
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore_list::create(&state.pool, &chore_list).await.unwrap();

    Redirect::to(&format!("/chore-lists/{}", chore_list.id))
}

#[derive(Template)]
#[template(path = "page/chore_list/update.jinja")]
struct UpdateTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_update_form(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Html(UpdateTemplate {chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    name: String,
}

pub async fn update(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.name = payload.name;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = Some(chrono::offset::Utc::now());

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    chore_list.date_deleted = None;

    chore_list::update(&state.pool, &chore_list).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}", chore_list.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/list.jinja")]
struct ChoreListTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
}

pub async fn view_chore_list(
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

    Ok(Html(ChoreListTemplate {chore_list, chores}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/chore/create.jinja")]
struct CreateChoreTemplate {
    chore_list: chore_list::ChoreList,
}

pub async fn view_create_chore_form(
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

    Ok(Html(CreateChoreTemplate {chore_list}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreateChorePayload {
    name: String,
    points: i32,
}

pub async fn create_chore(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
    Form(payload): Form<CreateChorePayload>,
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
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore::create(&state.pool, &chore).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/list.jinja")]
struct ActivityListTemplate {
    chore_list: chore_list::ChoreList,
    activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
    users: Vec<user::User>,
}

pub async fn view_activity_list(
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
    let activities = chore_activity::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let users = user::get_all(&state.pool).await.unwrap();

    Ok(Html(ActivityListTemplate {chore_list, activities, chores, users}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/create.jinja")]
struct CreateActivityTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    now: chrono::DateTime<chrono::Utc>
}

pub async fn view_create_activity_form(
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

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let now = chrono::offset::Utc::now();

    Ok(Html(CreateActivityTemplate {chore_list, chores, now}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreateActivityPayload {
    chore_id: Uuid,
    date: chrono::NaiveDate,
}

pub async fn create_activity(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Form(payload): Form<CreateActivityPayload>,
) -> Result<Redirect, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &chore_list_id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if chore_list.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = match chore::get_by_id(&state.pool, &payload.chore_id).await {
        Ok(chore) => chore,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(err) => panic!("{}", err),
    };

    if chore.chore_list_id != chore_list.id {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let activity = chore_activity::ChoreActivity {
        id: Uuid::now_v7(),
        chore_id: chore.id,
        user_id: auth_session.user_id,
        date: payload.date,
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore_activity::create(&state.pool, &activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities", chore_list.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/list_users.jinja")]
struct UserListTemplate {
    chore_list: chore_list::ChoreList,
    users: Vec<user::User>,
    scores_by_user: HashMap<Uuid, i32>,
}

pub async fn view_users_list(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let chore_list = match chore_list::get_by_id(&state.pool, &id).await {
        Ok(chore_list) => chore_list,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let users = user::get_all(&state.pool).await.unwrap();
    let scores_by_user = chore_list::get_score_per_user(&state.pool, &chore_list.id).await.unwrap();

    Ok(Html(UserListTemplate {chore_list, users, scores_by_user}.render().unwrap()))
}
