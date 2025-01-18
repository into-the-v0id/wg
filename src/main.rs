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
        .route("/people", get(application::person::view_list))
        .route("/people/create", get(application::person::view_create_form).post(application::person::create))
        .route("/people/{id}", get(application::person::view_detail))
        .route("/people/{id}/delete", post(application::person::delete))
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
