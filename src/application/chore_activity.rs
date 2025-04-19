use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore;
use crate::domain::chore_activity;
use crate::domain::chore_list;
use crate::domain::user;
use crate::{
    AppState,
    domain::value::{Date, DateTime, Uuid},
};
use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect},
};
use chrono::Days;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "page/chore_list/activity/list.jinja")]
struct ListTemplate<'a> {
    chore_list: chore_list::ChoreList,
    activities_by_date: Vec<(Date, Vec<&'a chore_activity::ChoreActivity>)>,
    deleted_activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
    users: Vec<user::User>,
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

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap();
    let users = user::get_all(&state.pool).await.unwrap();

    let (activities, deleted_activities): (Vec<_>, Vec<_>) = chore_activity::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap()
        .into_iter()
        .partition(|activity| !activity.is_deleted());
    let activities_by_date = chore_activity::group_and_sort_by_date(activities.iter().collect(), true);

    Ok(Html(
        ListTemplate {
            chore_list,
            activities_by_date,
            deleted_activities,
            chores,
            users,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/detail.jinja")]
struct DetailTemplate {
    activity: chore_activity::ChoreActivity,
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    user: user::User,
    auth_session: AuthenticationSession,
    allow_edit: bool,
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
    if chore.chore_list_id != chore_list_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let chore_list = chore_list::get_by_id(&state.pool, &chore.chore_list_id)
        .await
        .unwrap();
    let user = user::get_by_id(&state.pool, &activity.user_id)
        .await
        .unwrap();

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    let allow_edit = activity.date.as_ref() >= &min_date;

    Ok(Html(
        DetailTemplate {
            activity,
            chore,
            chore_list,
            user,
            auth_session,
            allow_edit,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/create.jinja")]
struct CreateTemplate {
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    min_date: Date,
    max_date: Date,
    now: DateTime,
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

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap();
    let min_date = Date::from((chrono::Utc::now() - Days::new(2)).date_naive());
    let max_date = Date::now();
    let now = DateTime::now();

    Ok(Html(
        CreateTemplate {
            chore_list,
            chores,
            min_date,
            max_date,
            now,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    chore_id: Uuid,
    date: Date,
    comment: String,
}

pub async fn create(
    Path(chore_list_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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
        id: Uuid::new(),
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

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/activities",
        chore_list.id
    )))
}

#[derive(Template)]
#[template(path = "page/chore_list/activity/update.jinja")]
struct UpdateTemplate {
    activity: chore_activity::ChoreActivity,
    chores: Vec<chore::Chore>,
    chore_list: chore_list::ChoreList,
    min_date: Date,
    max_date: Date,
}

pub async fn view_update_form(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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

    let chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
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

    let min_date = (chrono::Utc::now() - Days::new(2)).date_naive();
    if activity.date.as_ref() < &min_date {
        return Err(StatusCode::FORBIDDEN);
    }

    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap();
    let min_date = Date::from(min_date);
    let max_date = Date::now();

    Ok(Html(
        UpdateTemplate {
            activity,
            chores,
            chore_list,
            min_date,
            max_date,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct UpdatePayload {
    chore_id: Uuid,
    date: Date,
    comment: String,
}

pub async fn update(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
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

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/activities/{}",
        chore_list.id, activity.id
    )))
}

pub async fn delete(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    auth_session: AuthenticationSession,
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

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
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

    activity.date_deleted = Some(DateTime::now());

    chore_activity::update(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/activities/{}",
        chore_list.id, activity.id
    )))
}

pub async fn restore(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Redirect, StatusCode> {
    let mut activity = match chore_activity::get_by_id(&state.pool, &id).await {
        Ok(chore_activity) => chore_activity,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };
    if !activity.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut chore = chore::get_by_id(&state.pool, &activity.chore_id)
        .await
        .unwrap();
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

    activity.date_deleted = None;

    chore_activity::update(&state.pool, &activity)
        .await
        .unwrap();

    chore::update_next_due_date(&mut chore, &state.pool, true)
        .await
        .unwrap();

    Ok(Redirect::to(&format!(
        "/chore-lists/{}/activities/{}",
        chore_list.id, activity.id
    )))
}
