use askama::Template;
use axum::response::Html;

use crate::domain::authentication_session::AuthenticationSession;

#[derive(Template)]
#[template(path = "page/settings.jinja")]
struct SettingsTemplate {
    auth_session: AuthenticationSession,
}

pub async fn view(auth_session: AuthenticationSession) -> Html<String> {
    Html(SettingsTemplate { auth_session }.render().unwrap())
}
