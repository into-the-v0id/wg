use wg_core::model::user;
use wg_core::model::chore;
use wg_core::model::chore::ChoreId;
use wg_core::model::chore_activity;
use wg_core::model::chore_activity::ChoreActivityId;
use wg_core::model::chore_list;
use wg_core::model::chore_list::ChoreListId;
use wg_core::model::user::UserId;
use crate::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum::{
    extract::Path,
    http::StatusCode,
};
use std::sync::Arc;

pub struct Chore(pub chore::Chore);

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChorePathData {
    chore_id: ChoreId,
}

impl FromRequestParts<Arc<AppState>> for Chore {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<ChorePathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let chore = match chore::get_by_id(&state.pool, &path_data.chore_id).await {
            Ok(chore) => chore,
            Err(wg_core::db::sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(Chore(chore))
    }
}

pub struct ChoreActivity(pub chore_activity::ChoreActivity);

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChoreActivityPathData {
    chore_activity_id: ChoreActivityId,
}

impl FromRequestParts<Arc<AppState>> for ChoreActivity {
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
            Err(wg_core::db::sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(ChoreActivity(activity))
    }
}

pub struct ChoreList(pub chore_list::ChoreList);

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct ChoreListPathData {
    chore_list_id: ChoreListId,
}

impl FromRequestParts<Arc<AppState>> for ChoreList {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<ChoreListPathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let chore_list = match chore_list::get_by_id(&state.pool, &path_data.chore_list_id).await {
            Ok(chore_list) => chore_list,
            Err(wg_core::db::sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(ChoreList(chore_list))
    }
}

pub struct User(pub user::User);

#[derive(Debug, Copy, Clone, serde::Deserialize)]
struct UserPathData {
    user_id: UserId,
}

impl FromRequestParts<Arc<AppState>> for User {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let path_data = match parts.extract::<Path<UserPathData>>().await {
            Ok(path_data) => path_data,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let user = match user::get_by_id(&state.pool, &path_data.user_id).await {
            Ok(user) => user,
            Err(wg_core::db::sqlx::Error::RowNotFound) => return Err(StatusCode::NOT_FOUND),
            Err(err) => panic!("{}", err),
        };

        Ok(User(user))
    }
}
