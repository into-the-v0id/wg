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

pub mod template;
pub mod handler;
pub mod extractor;

use extractor::{language::Language, theme::Theme};
use axum::{
    RequestExt, Router,
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_extra::routing::RouterExt;
use tokio::{net::TcpListener, task_local};
use tokio_util::sync::CancellationToken;
use wg_core::{db::Pool, model::authentication_session::AuthenticationSession};
use std::{any::Any, sync::Arc};
use tower_http::{
    catch_panic::CatchPanicLayer,
    request_id,
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Instrument;
use tracing::Level;
use fluent_static::{message_bundle, MessageBundle};

use crate::extractor::authentication::AuthSession;

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
    pub static THEME: Theme;
}

pub struct AppState {
    pub pool: Pool,
}

pub fn make_router(state: AppState) -> Router {
    let state = Arc::new(state);

    Router::new()
        .route("/", get(handler::entry::redirect))
        .route("/healthz", get(handler::health::check))

        // Authentication
        .typed_get(handler::authentication::view_login_form)
        .typed_post(handler::authentication::login)
        .typed_post(handler::authentication::logout)

        // Settings
        .typed_get(handler::settings::view)
        .typed_get(handler::settings::view_appearance_form)
        .typed_post(handler::settings::update_appearance)

        // User
        .typed_get(handler::user::view_list)
        .typed_get(handler::user::view_create_form)
        .typed_post(handler::user::create)
        .typed_get(handler::user::view_detail)
        .typed_get(handler::user::view_update_form)
        .typed_post(handler::user::update)
        .typed_post(handler::user::delete)
        .typed_post(handler::user::restore)

        // Chore List
        .typed_get(handler::chore_list::view_list)
        .typed_get(handler::chore_list::view_create_form)
        .typed_post(handler::chore_list::create)
        .typed_get(handler::chore_list::view_settings)
        .typed_get(handler::chore_list::view_update_form)
        .typed_post(handler::chore_list::update)
        .typed_post(handler::chore_list::delete)
        .typed_post(handler::chore_list::restore)

        // Chore
        .typed_get(handler::chore::view_list)
        .typed_get(handler::chore::view_create_form)
        .typed_post(handler::chore::create)
        .typed_get(handler::chore::view_detail)
        .typed_get(handler::chore::view_update_form)
        .typed_post(handler::chore::update)
        .typed_post(handler::chore::delete)
        .typed_post(handler::chore::restore)
        .typed_get(handler::chore::view_activity_list)

        // Chore Activity
        .typed_get(handler::chore_activity::view_list)
        .typed_get(handler::chore_activity::view_create_form)
        .typed_post(handler::chore_activity::create)
        .typed_get(handler::chore_activity::view_detail)
        .typed_get(handler::chore_activity::view_update_form)
        .typed_post(handler::chore_activity::update)
        .typed_post(handler::chore_activity::delete)
        .typed_post(handler::chore_activity::restore)

        // Chore List User
        .typed_get(handler::chore_list_user::view_list)
        .typed_get(handler::chore_list_user::view_detail)
        .typed_get(handler::chore_list_user::view_activity_list)

        // Legal
        .typed_get(handler::legal::view_privacy_policy)

        .fallback_service(get(handler::assets::serve))
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
                let body = Body::from(template::page::error::http_error(status_code, request_id).into_string());

                return Response::from_parts(response_parts, body);
            }

            response
        }))
        .layer(axum::middleware::from_fn(async |mut request: axum::extract::Request, next: Next| -> Response {
            let theme = request.extract_parts::<Theme>().await.unwrap();

            THEME.scope(theme, next.run(request)).await
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
            state.clone(),
            async |State(state): State<Arc<AppState>>, mut request: axum::extract::Request, next: Next| -> Response {
                let user_id = match request.extract_parts_with_state::<AuthSession, _>(&state).await {
                    Ok(AuthSession(AuthenticationSession {user_id, ..})) => user_id,
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
        .with_state(state)
}

pub async fn start(listener: TcpListener, router: Router, cancel_token: CancellationToken) {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move { cancel_token.cancelled().await })
        .await
        .unwrap();
}
