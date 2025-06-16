use axum::http::StatusCode;
use maud::{html, Markup};
use crate::web::{application::authentication::LoginPath, template::layout};

pub fn http_error(
    status_code: StatusCode,
    request_id: Option<String>,
) -> Markup {
    let error_name = status_code.canonical_reason().unwrap_or("Unknown Error");

    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title(error_name)
            .headline(error_name)
            .teaser(&format!("ERROR {}", status_code.as_u16()))
            .back_url("/")
            .head(html! {
                @if status_code.as_u16() == 401 {
                    meta http-equiv="refresh" content={ "0; url=" (LoginPath) };
                }
            })
            .build(),
        html! {
            @if let Some(request_id) = request_id {
                small.text-muted {
                    "Request ID: " (request_id)
                }
            }
        },
    )
}
