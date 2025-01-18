pub mod domain;
pub mod application;

use std::sync::Arc;
use axum::{
    http::{HeaderName, Request}, routing::{get, post}, Router
};
use sqlx::migrate::MigrateDatabase;
use tower_http::{request_id, trace::TraceLayer};

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
        .route("/users", get(application::user::view_list))
        .route("/users/create", get(application::user::view_create_form).post(application::user::create))
        .route("/users/{id}", get(application::user::view_detail))
        .route("/users/{id}/delete", post(application::user::delete))
        .route("/users/{id}/restore", post(application::user::restore))
        .with_state(app_state)
        .layer(request_id::PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the request id as generated.
                let request_id = request.headers().get("x-request-id");

                match request_id {
                    Some(request_id) => tracing::info_span!(
                        "http_request",
                        request_id = ?request_id,
                    ),
                    None => {
                        tracing::error!("could not extract request_id");
                        tracing::info_span!("http_request")
                    }
                }
            })
        )
        .layer(request_id::SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            request_id::MakeRequestUuid,
        ));

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
