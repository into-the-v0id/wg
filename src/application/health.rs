use std::sync::Arc;
use axum::{extract::State, http::StatusCode};
use crate::AppState;

pub async fn check(State(state): State<Arc<AppState>>) -> StatusCode {
    let result = match sqlx::query_as::<_, (i32,)>("SELECT 1 + 1").fetch_one(&state.pool).await {
        Ok(result) => result.0,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if result != 2 {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
