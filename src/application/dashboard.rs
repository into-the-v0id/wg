use crate::domain::authentication_session::AuthenticationSession;
use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "page/dashboard.jinja")]
struct DashboardTemplate();

pub async fn view(_auth_session: AuthenticationSession) -> Html<String> {
    Html(DashboardTemplate().render().unwrap())
}
