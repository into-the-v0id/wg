use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "page/dashboard.jinja")]
struct DashboardTemplate();

pub async fn view() -> Html<String> {
    Html(DashboardTemplate().render().unwrap())
}
