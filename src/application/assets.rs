use std::collections::HashMap;
use axum::{extract::Query, http::{header, StatusCode, Uri}, response::IntoResponse};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "static"]
struct Asset;

pub async fn serve(
    uri: Uri,
    Query(query_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    let file = match Asset::get(path) {
        Some(file) => file,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mime_type = mime_guess::from_path(path).first_or_octet_stream();

    let cache_control = if query_params.get("hash").unwrap_or(&"".to_string()) != "" {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=0, must-revalidate"
    };

    (
        [
            (header::CONTENT_TYPE, mime_type.as_ref()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, cache_control),
        ],
        file.data,
    ).into_response()
}

pub fn get_url(path: &str) -> Option<String> {
    let path = path.trim_start_matches('/');
    let file = Asset::get(path)?;
    let hash = const_hex::encode(file.metadata.sha256_hash());

    Some(format!("/{path}?hash={hash}"))
}
