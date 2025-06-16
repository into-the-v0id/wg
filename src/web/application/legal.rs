use axum_extra::routing::TypedPath;
use maud::Markup;
use crate::web::template;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/legal/privacy-policy")]
pub struct PrivacyPolicyPath;

pub async fn view_privacy_policy(_path: PrivacyPolicyPath) -> Markup {
    template::page::legal::privacy_policy()
}
