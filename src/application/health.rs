use std::sync::Arc;
use axum::{extract::State, http::StatusCode};
use sqlx::Connection;
use crate::AppState;

pub async fn check(State(state): State<Arc<AppState>>) -> StatusCode {
    if state.pool.acquire().await.unwrap().as_mut().ping().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
