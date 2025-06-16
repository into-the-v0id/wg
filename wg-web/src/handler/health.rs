use crate::AppState;
use axum::{extract::State, http::StatusCode};
use sqlx::Connection;
use std::sync::Arc;

pub async fn check(State(state): State<Arc<AppState>>) -> StatusCode {
    if state
        .pool
        .acquire()
        .await
        .unwrap()
        .as_mut()
        .ping()
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
