// Copyright (C) Oliver Amann
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod domain;
pub mod application;
pub mod templates;

use application::language::Language;
use axum::{
    RequestExt, Router,
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use domain::{
    authentication_session::AuthenticationSession,
    value::{DateTime, PasswordHash, Uuid},
};
use sqlx::migrate::MigrateDatabase;
use std::{any::Any, sync::Arc};
use tokio::signal;
use tokio::task_local;
use tower_http::{
    catch_panic::CatchPanicLayer,
    request_id,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Instrument;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use fluent_static::message_bundle;
use fluent_static::MessageBundle;

#[message_bundle(
    resources = [
        ("translations/en.ftl", "en"),
        ("translations/de.ftl", "de"),
    ],
    default_language = "en",
)]
pub struct Translations;

task_local! {
    pub static LANGUAGE: Language;
    pub static TRANSLATIONS: Translations;
}

pub struct AppState {
    pub pool: sqlx::sqlite::SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let app_state = Arc::new(AppState {
        pool: create_db_pool().await,
    });

    sqlx::migrate!().run(&app_state.pool).await.unwrap();

    create_user_if_necessary(&app_state.pool).await;

    let router = Router::new()
        .route("/login", get(application::authentication::view_login_form).post(application::authentication::login))
        .route("/logout", post(application::authentication::logout))
        .route("/", get(application::entry::redirect))
        .route("/settings", get(application::settings::view))
        .route("/settings/appearance", get(application::settings::view_appearance_form).post(application::settings::update_appearance))
        .route("/users", get(application::user::view_list))
        .route("/users/create", get(application::user::view_create_form).post(application::user::create))
        .route("/users/{user_id}", get(application::user::view_detail))
        .route("/users/{user_id}/update", get(application::user::view_update_form).post(application::user::update))
        .route("/users/{user_id}/delete", post(application::user::delete))
        .route("/users/{user_id}/restore", post(application::user::restore))
        .route("/chore-lists", get(application::chore_list::view_list))
        .route("/chore-lists/create", get(application::chore_list::view_create_form).post(application::chore_list::create))
        .route("/chore-lists/{chore_list_id}/settings", get(application::chore_list::view_settings))
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
        .route("/chore-lists/{chore_list_id}/users", get(application::chore_list_user::view_list))
        .route("/chore-lists/{chore_list_id}/users/{user_id}", get(application::chore_list_user::view_detail))
        .route("/chore-lists/{chore_list_id}/users/{user_id}/activities", get(application::chore_list_user::view_activity_list))
        .route("/legal/privacy-policy", get(application::legal::view_privacy_policy))
        .route("/healthz", get(application::health::check))
        .fallback_service(get(application::assets::serve))
        .layer(axum::middleware::from_fn(async |request: axum::extract::Request, next: Next| -> Response {
            let request_id = request.headers().get("x-request-id")
                .map(|v| v.to_str().unwrap().to_string());

            let response = next.run(request).await;

            let status_code = response.status();
            if status_code.is_client_error() || status_code.is_server_error() {
                let (mut response_parts, _body) = response.into_parts();

                response_parts.headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8")
                );
                response_parts.headers.remove(header::CONTENT_LENGTH);
                response_parts.headers.remove(header::CONTENT_ENCODING);
                let body = Body::from(templates::page::error::http_error(status_code, request_id).into_string());

                return Response::from_parts(response_parts, body);
            }

            response
        }))
        .layer(axum::middleware::from_fn(async |mut request: axum::extract::Request, next: Next| -> Response {
            let language = request.extract_parts::<Language>().await.unwrap();
            LANGUAGE.scope(language, async {
                let translations = Translations::get(&language.to_string()).unwrap();
                TRANSLATIONS.scope(translations, async {
                    next.run(request).await
                }).await
            }).await
        }))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
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
            HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline' 'self'; img-src data: 'self'; script-src 'self'; frame-ancestors 'none'; form-action 'self'; manifest-src 'self';"),
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
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            async |State(state): State<Arc<AppState>>, mut request: axum::extract::Request, next: Next| -> Response {
                let user_id = match request.extract_parts_with_state::<AuthenticationSession, _>(&state).await {
                    Ok(AuthenticationSession {user_id, ..}) => user_id,
                    Err(_) => return next.run(request).await,
                };

                let span = tracing::error_span!(
                    "auth_session",
                    user_id = %user_id,
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
                request_id = request_id.to_str().unwrap(),
            );

            next.run(request).instrument(span).await
        }))
        .layer(request_id::PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")))
        .layer(request_id::SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            request_id::MakeRequestUuid,
        ))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::ERROR)))
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

    let port = std::env::var("PORT")
        .map(|raw_port| raw_port.parse::<i32>().unwrap())
        .unwrap_or(3000);
    let address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Listening on http://{} ...", listener.local_addr().unwrap());

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn create_db_pool() -> sqlx::sqlite::SqlitePool {
    let db_file = std::env::var("DB_FILE").unwrap_or(String::from("./data/sqlite.db"));
    let db_url = format!("sqlite:{}", db_file);

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        tracing::info!("Creating database {}", &db_url);
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    sqlx::sqlite::SqlitePool::connect(&db_url).await.unwrap()
}

/// If no users exist, create a user and print ist credentials.
/// Mainly used for new-installs without an existing DB.
async fn create_user_if_necessary(pool: &sqlx::sqlite::SqlitePool) {
    let users = domain::user::get_all(pool).await.unwrap();
    if !users.is_empty() {
        return;
    }

    let mut plain_password_buf = [0u8; 8];
    getrandom::getrandom(&mut plain_password_buf).unwrap();
    let plain_password = const_hex::encode(plain_password_buf);

    let user = domain::user::User {
        id: Uuid::new(),
        name: "Admin".to_string(),
        handle: "admin".to_string(),
        password_hash: PasswordHash::from_plain_password(plain_password.clone().into()),
        date_created: DateTime::now(),
        date_deleted: None,
    };
    domain::user::create(pool, &user).await.unwrap();

    println!(
        "Created user with handle '{}' and password '{}'",
        user.handle, plain_password
    );
}
