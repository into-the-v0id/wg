pub mod domain;
pub mod application;

use std::{any::Any, sync::Arc};
use axum::{
    http::{HeaderName, HeaderValue, Request, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Router
};
use sqlx::migrate::MigrateDatabase;
use tower_http::{catch_panic::CatchPanicLayer, request_id, trace::TraceLayer};

pub struct AppState {
    pub pool: sqlx::sqlite::SqlitePool
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState {
        pool: create_db_pool().await,
    });

    sqlx::migrate!()
        .run(&app_state.pool)
        .await
        .unwrap();

    let router = Router::new()
        .route("/", get(application::dashboard::view))
        .route("/users", get(application::user::view_list))
        .route("/users/create", get(application::user::view_create_form).post(application::user::create))
        .route("/users/{id}", get(application::user::view_detail))
        .route("/users/{id}/update", get(application::user::view_update_form).post(application::user::update))
        .route("/users/{id}/delete", post(application::user::delete))
        .route("/users/{id}/restore", post(application::user::restore))
        .route("/chore-lists", get(application::chore_list::view_list))
        .route("/chore-lists/create", get(application::chore_list::view_create_form).post(application::chore_list::create))
        .route("/chore-lists/{id}", get(application::chore_list::view_detail))
        .route("/chore-lists/{id}/update", get(application::chore_list::view_update_form).post(application::chore_list::update))
        .route("/chore-lists/{id}/delete", post(application::chore_list::delete))
        .route("/chore-lists/{id}/restore", post(application::chore_list::restore))
        .route("/chore-lists/{id}/chores", get(application::chore_list::view_chore_list))
        .route("/chore-lists/{id}/chores/create", get(application::chore_list::view_create_chore_form).post(application::chore_list::create_chore))
        .route("/chore-lists/{id}/activities", get(application::chore_list::view_activity_list))
        .route("/chore-lists/{id}/activities/create", get(application::chore_list::view_create_activity_form).post(application::chore_list::create_activity))
        .route("/chore-lists/{id}/users", get(application::chore_list::view_users_list))
        .route("/chores/{id}", get(application::chore::view_detail))
        .route("/chores/{id}/update", get(application::chore::view_update_form).post(application::chore::update))
        .route("/chores/{id}/delete", post(application::chore::delete))
        .route("/chores/{id}/restore", post(application::chore::restore))
        .route("/chores/{id}/activities", get(application::chore::view_activity_list))
        .route("/chore-activities/{id}", get(application::chore_activity::view_detail))
        .route("/chore-activities/{id}/update", get(application::chore_activity::view_update_form).post(application::chore_activity::update))
        .route("/chore-activities/{id}/delete", post(application::chore_activity::delete))
        .route("/chore-activities/{id}/restore", post(application::chore_activity::restore))
        .with_state(app_state)
        .layer(request_id::PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request.headers().get("x-request-id")
                    .map(|v| v.clone())
                    .unwrap_or(HeaderValue::from_static("None"));

                let user_agent = request.headers().get("user-agent")
                    .map(|v| v.clone())
                    .unwrap_or(HeaderValue::from_static("None"));

                tracing::info_span!(
                    "http_request",
                    request_id = ?request_id,
                    user_agent = ?user_agent,
                )
            })
        )
        .layer(request_id::SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            request_id::MakeRequestUuid,
        ))
        .layer(CatchPanicLayer::custom(|err: Box<dyn Any + Send + 'static>| {
            if let Some(s) = err.downcast_ref::<String>() {
                tracing::error!("Service panicked: {}", s);
            } else if let Some(s) = err.downcast_ref::<&str>() {
                tracing::error!("Service panicked: {}", s);
            } else {
                tracing::error!("Service panicked but `CatchPanic` was unable to downcast the panic info");
            };

            handle_error(StatusCode::INTERNAL_SERVER_ERROR)
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}

async fn create_db_pool() -> sqlx::sqlite::SqlitePool {
    let db_url = "sqlite:./sqlite.db";

    if ! sqlx::Sqlite::database_exists(db_url).await.unwrap_or(false) {
        tracing::info!("Creating database {}", db_url);
        sqlx::Sqlite::create_database(db_url).await.unwrap();
    }

    sqlx::sqlite::SqlitePool::connect(db_url).await.unwrap()
}

fn handle_error(status: StatusCode) -> Response {
    (status, status.canonical_reason().unwrap_or("Unknown Error")).into_response()
}
