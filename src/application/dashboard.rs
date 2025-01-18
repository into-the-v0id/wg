use askama::Template;
use axum::response::Html;
use super::authentication::AuthSession;

#[derive(Template)]
#[template(path = "page/dashboard.jinja")]
struct DashboardTemplate();

pub async fn view(_auth_session: AuthSession) -> Html<String> {
    Html(DashboardTemplate().render().unwrap())
}
