pub mod domain;
pub mod application;

use std::{any::Any, sync::Arc};
use askama::Template;
use axum::{
    body::Body, extract::State, http::{header, HeaderName, HeaderValue, StatusCode}, middleware::Next, response::{IntoResponse, Response}, routing::{get, post}, RequestExt, Router
};
use sqlx::migrate::MigrateDatabase;
use tokio::sync::Mutex;
use tower_http::{catch_panic::CatchPanicLayer, request_id, set_header::SetResponseHeaderLayer, trace::{DefaultMakeSpan, TraceLayer}};
use tracing::Level;
use tracing::Instrument;

pub struct AppState {
    pub pool: sqlx::sqlite::SqlitePool,
    pub auth_sessions: Mutex<Vec<application::authentication::AuthSession>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState {
        pool: create_db_pool().await,
        auth_sessions: Mutex::new(Vec::new()),
    });

    sqlx::migrate!()
        .run(&app_state.pool)
        .await
        .unwrap();

    let router = Router::new()
        .route("/login", get(application::authentication::view_login_form).post(application::authentication::login))
        .route("/", get(application::dashboard::view))
        .route("/users", get(application::user::view_list))
        .route("/users/create", get(application::user::view_create_form).post(application::user::create))
        .route("/users/{user_id}", get(application::user::view_detail))
        .route("/users/{user_id}/update", get(application::user::view_update_form).post(application::user::update))
        .route("/users/{user_id}/delete", post(application::user::delete))
        .route("/users/{user_id}/restore", post(application::user::restore))
        .route("/chore-lists", get(application::chore_list::view_list))
        .route("/chore-lists/create", get(application::chore_list::view_create_form).post(application::chore_list::create))
        .route("/chore-lists/{chore_list_id}", get(application::chore_list::view_detail))
        .route("/chore-lists/{chore_list_id}/update", get(application::chore_list::view_update_form).post(application::chore_list::update))
        .route("/chore-lists/{chore_list_id}/delete", post(application::chore_list::delete))
        .route("/chore-lists/{chore_list_id}/restore", post(application::chore_list::restore))
        .route("/chore-lists/{chore_list_id}/chores", get(application::chore::view_list))
        .route("/chore-lists/{chore_list_id}/chores/create", get(application::chore::view_create_form).post(application::chore::create))
        .route("/chore-lists/{chore_list_id}/chores/{chore_id}", get(application::chore::view_detail))
        .route("/chore-lists/{chore_list_id}/chores/{chore_id}/update", get(application::chore::view_update_form).post(application::chore::update))
        .route("/chore-lists/{chore_list_id}/chores/{chore_id}/delete", post(application::chore::delete))
        .route("/chore-lists/{chore_list_id}/chores/{chore_id}/restore", post(application::chore::restore))
        .route("/chore-lists/{chore_list_id}/chores/{chore_id}/activities", get(application::chore::view_activity_list))
        .route("/chore-lists/{chore_list_id}/activities", get(application::chore_activity::view_list))
        .route("/chore-lists/{chore_list_id}/activities/create", get(application::chore_activity::view_create_form).post(application::chore_activity::create))
        .route("/chore-lists/{chore_list_id}/activities/{chore_activity_id}", get(application::chore_activity::view_detail))
        .route("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/update", get(application::chore_activity::view_update_form).post(application::chore_activity::update))
        .route("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/delete", post(application::chore_activity::delete))
        .route("/chore-lists/{chore_list_id}/activities/{chore_activity_id}/restore", post(application::chore_activity::restore))
        .route("/chore-lists/{chore_list_id}/users", get(application::chore_list::view_users_list))
        .layer(axum::middleware::from_fn(async |request: axum::extract::Request, next: Next| -> Response {
            let response = next.run(request).await;

            let status_code = response.status();
            if status_code.is_client_error() || status_code.is_server_error() {
                let (mut response_parts, _body) = response.into_parts();

                response_parts.headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref())
                );
                let body = Body::from(ErrorTemplate {status_code}.render().unwrap());

                return Response::from_parts(response_parts, body);
            }

            return response;
        }))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, max-age=0, must-revalidate"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline' https://cdn.jsdelivr.net/npm/@picocss/pico@2.0.6/css/pico.min.css; img-src data:; frame-ancestors 'none'; form-action 'self';"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("interest-cohort=()"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow"),
        ))
        .layer(request_id::PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::ERROR)))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            async |State(state): State<Arc<AppState>>, mut request: axum::extract::Request, next: Next| -> Response {
                let user_id = match request.extract_parts_with_state::<application::authentication::AuthSession, _>(&state).await {
                    Ok(application::authentication::AuthSession {user_id, ..}) => user_id,
                    Err(_) => return next.run(request).await,
                };

                let span = tracing::error_span!(
                    "auth_session",
                    user_id = ?user_id,
                );

                next.run(request).instrument(span).await
            }
        ))
        .layer(axum::middleware::from_fn(async |request: axum::extract::Request, next: Next| -> Response {
            let request_id = match request.headers().get("x-request-id") {
                Some(request_id) => request_id,
                None => return next.run(request).await,
            };

            let span = tracing::error_span!(
                "request_id",
                request_id = ?request_id,
            );

            next.run(request).instrument(span).await
        }))
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

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::INTERNAL_SERVER_ERROR.canonical_reason().unwrap_or("Unknown Error")
            ).into_response()
        }))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}

async fn create_db_pool() -> sqlx::sqlite::SqlitePool {
    let db_url = "sqlite:./data/sqlite.db";

    if ! sqlx::Sqlite::database_exists(db_url).await.unwrap_or(false) {
        tracing::info!("Creating database {}", db_url);
        sqlx::Sqlite::create_database(db_url).await.unwrap();
    }

    sqlx::sqlite::SqlitePool::connect(db_url).await.unwrap()
}

#[derive(Template)]
#[template(path = "page/error.jinja")]
struct ErrorTemplate {
    status_code: StatusCode,
}
