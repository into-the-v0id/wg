use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::chore_list;
use crate::domain::user;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/chore_list/activity/list.jinja")]
struct ListTemplate {
    chore_list: chore_list::ChoreList,
    activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
    users: Vec<user::User>,
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
    let activities = chore_activity::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let users = user::get_all(&state.pool).await.unwrap();

    Ok(Html(ListTemplate {chore_list, activities, chores, users}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/detail.jinja")]
struct DetailTemplate {
    activity: chore_activity::ChoreActivity,
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    user: user::User,
    auth_session: AuthSession,
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND)
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id).await.unwrap();
    let user = user::get_by_id(&state.pool, &activity.user_id).await.unwrap();

    Ok(Html(DetailTemplate {activity, chore, chore_list, user, auth_session}.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/create.jinja")]
struct CreateTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    now: chrono::DateTime<chrono::Utc>
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

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let now = chrono::offset::Utc::now();

    Ok(Html(CreateTemplate {chore_list, chores, now}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    chore_id: Uuid,
    date: chrono::NaiveDate,
    comment: String,
}

pub async fn create(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
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
        comment: match payload.comment.trim() {
            "" => None,
            comment => Some(comment.to_string()),
        },
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    chore_activity::create(&state.pool, &activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities", chore_list.id)))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/update.jinja")]
struct UpdateTemplate {
    activity: chore_activity::ChoreActivity,
    chores: Vec<chore::Chore>,
    chore_list: chore_list::ChoreList,
    now: chrono::DateTime<chrono::Utc>
}

pub async fn view_update_form(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
) -> Result<Html<String>, StatusCode> {
    let activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
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

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id).await.unwrap();
    let now = chrono::offset::Utc::now();

    Ok(Html(UpdateTemplate {activity, chores, chore_list, now}.render().unwrap()))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    chore_id: Uuid,
    date: chrono::NaiveDate,
    comment: String,
}

pub async fn update(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    let mut activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
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

    activity.chore_id = payload.chore_id;
    activity.date = payload.date;
    activity.comment = match payload.comment.trim() {
        "" => None,
        comment => Some(comment.to_string()),
    };

    chore_activity::update(&state.pool, &activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities/{}", chore_list.id, activity.id)))
}

pub async fn delete(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if activity.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
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

    activity.date_deleted = Some(chrono::offset::Utc::now());

    chore_activity::update(&state.pool, &activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities/{}", chore_list.id, activity.id)))
}

pub async fn restore(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthSession,
) -> Result<Redirect, StatusCode> {
    let mut activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let chore = chore::get_by_id(&state.pool, &activity.chore_id).await.unwrap();
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

    activity.date_deleted = None;

    chore_activity::update(&state.pool, &activity).await.unwrap();

    Ok(Redirect::to(&format!("/chore-lists/{}/activities/{}", chore_list.id, activity.id)))
}
