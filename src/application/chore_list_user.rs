use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore;
use crate::domain::chore_activity;
use crate::domain::chore_list;
use crate::domain::user;
use crate::{
    AppState,
    domain::value::{Date, Uuid},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use std::sync::Arc;

#[derive(Template)]
#[template(path = "page/chore_list/user/list.jinja")]
struct ListTemplate {
    chore_list: chore_list::ChoreList,
    users: Vec<user::User>,
    deleted_users: Vec<user::User>,
    scores_by_user: Vec<(Uuid, i32)>,
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

    let (users, deleted_users) = user::get_all(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .partition(|user| !user.is_deleted());
    let scores_by_user = chore_list::get_score_per_user(&state.pool, &chore_list)
        .await
        .unwrap();

    Ok(Html(
        ListTemplate {
            chore_list,
            users,
            deleted_users,
            scores_by_user,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Template)]
#[template(path = "page/chore_list/user/detail.jinja")]
struct DetailTemplate {
    user: user::User,
    chore_list: chore_list::ChoreList,
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore_list = chore_list::get_by_id(&state.pool, &chore_list_id)
        .await
        .unwrap();

    Ok(Html(DetailTemplate { user, chore_list }.render().unwrap()))
}

#[derive(Template)]
#[template(path = "page/chore_list/user/list_activities.jinja")]
struct ActivityListTemplate<'a> {
    user: user::User,
    chore_list: chore_list::ChoreList,
    activities_by_date: Vec<(Date, Vec<&'a chore_activity::ChoreActivity>)>,
    deleted_activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
}

pub async fn view_activity_list(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Html<String>, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore_list = chore_list::get_by_id(&state.pool, &chore_list_id)
        .await
        .unwrap();
    let chores = chore::get_all_for_chore_list(&state.pool, &chore_list.id)
        .await
        .unwrap();

    let (activities, deleted_activities): (Vec<_>, Vec<_>) = chore_activity::get_all_for_chore_list_and_user(&state.pool, &chore_list_id, &user.id)
        .await
        .unwrap()
        .into_iter()
        .partition(|activity| !activity.is_deleted());
    let activities_by_date = chore_activity::group_and_sort_by_date(activities.iter().collect(), true);

    Ok(Html(
        ActivityListTemplate {
            user,
            chore_list,
            activities_by_date,
            deleted_activities,
            chores,
        }
        .render()
        .unwrap(),
    ))
}
