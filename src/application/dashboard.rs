use askama::Template;
use axum::response::Html;
use crate::domain::authentication_session::AuthenticationSession;

#[derive(Template)]
#[template(path = "page/dashboard.jinja")]
struct DashboardTemplate();

pub async fn view(_auth_session: AuthenticationSession) -> Html<String> {
    Html(DashboardTemplate().render().unwrap())
}
