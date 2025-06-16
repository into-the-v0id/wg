use wg_core::model::chore;
use wg_core::model::chore_activity;
use wg_core::model::chore_list;
use wg_core::model::chore_list::ChoreListId;
use wg_core::model::user;
use wg_core::model::user::UserId;
use crate::extractor::authentication::AuthSession;
use crate::extractor::model::ChoreList;
use crate::extractor::model::User;
use crate::template;
use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
};
use axum_extra::routing::TypedPath;
use maud::Markup;
use std::sync::Arc;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/users")]
pub struct ChoreListUserIndexPath {
    pub chore_list_id: ChoreListId,
}

pub async fn view_list(
    _path: ChoreListUserIndexPath,
    ChoreList(chore_list): ChoreList,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (all_users, scores_by_user) = tokio::try_join!(
        user::get_all(&state.pool),
        chore_list::get_score_per_user(&state.pool, &chore_list),
    ).unwrap();

    let (users, deleted_users) = all_users
        .into_iter()
        .partition(|user| !user.is_deleted());

    Ok(template::page::chore_list::user::list(
        chore_list,
        users,
        deleted_users,
        scores_by_user,
    ))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/users/{user_id}")]
pub struct ChoreListUserDetailPath {
    pub chore_list_id: ChoreListId,
    pub user_id: UserId,
}

pub async fn view_detail(
    _path: ChoreListUserDetailPath,
    ChoreList(chore_list): ChoreList,
    User(user): User,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    Ok(template::page::chore_list::user::detail(user, chore_list))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/chore-lists/{chore_list_id}/users/{user_id}/activities")]
pub struct ChoreListUserActivitiesPath {
    pub chore_list_id: ChoreListId,
    pub user_id: UserId,
}

pub async fn view_activity_list(
    _path: ChoreListUserActivitiesPath,
    ChoreList(chore_list): ChoreList,
    User(user): User,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (chores, all_activities) = tokio::try_join!(
        chore::get_all_for_chore_list(&state.pool, &chore_list.id),
        chore_activity::get_all_for_chore_list_and_user(&state.pool, &chore_list.id, &user.id),
    ).unwrap();

    let (activities, deleted_activities): (Vec<_>, Vec<_>) = all_activities
        .into_iter()
        .partition(|activity| !activity.is_deleted());
    let activities_by_date = chore_activity::group_and_sort_by_date(activities.iter().collect(), true);

    Ok(template::page::chore_list::user::list_activities(
        user,
        chore_list,
        activities_by_date,
        deleted_activities,
        chores,
    ))
}
