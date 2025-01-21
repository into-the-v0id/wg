use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "page/legal/privacy_policy.jinja")]
struct PrivacyPolicyTemplate();

pub async fn view_privacy_policy() -> Html<String> {
    Html(PrivacyPolicyTemplate().render().unwrap())
}
