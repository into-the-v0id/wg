use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore;
use crate::domain::chore_activity;
use crate::domain::chore_list;
use crate::domain::user;
use crate::templates;
use crate::{
    AppState,
    domain::value::Uuid,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
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

    let (users, deleted_users) = user::get_all(&state.pool)
        .await
        .unwrap()
        .into_iter()
        .partition(|user| !user.is_deleted());
    let scores_by_user = chore_list::get_score_per_user(&state.pool, &chore_list)
        .await
        .unwrap();

    Ok(templates::page::chore_list::user::list(
        chore_list,
        users,
        deleted_users,
        scores_by_user,
    ))
}

pub async fn view_detail(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
    let user = match user::get_by_id(&state.pool, &id).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
        Err(err) => panic!("{}", err),
    };

    let chore_list = chore_list::get_by_id(&state.pool, &chore_list_id)
        .await
        .unwrap();

    Ok(templates::page::chore_list::user::detail(user, chore_list))
}

pub async fn view_activity_list(
    Path((chore_list_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _auth_session: AuthenticationSession,
) -> Result<Markup, StatusCode> {
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

    Ok(templates::page::chore_list::user::list_activities(
        user,
        chore_list,
        activities_by_date,
        deleted_activities,
        chores,
    ))
}
