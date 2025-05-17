use axum_extra::routing::TypedPath;
use maud::Markup;
use crate::templates;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/legal/privacy-policy")]
pub struct PrivacyPolicyPath;

pub async fn view_privacy_policy(_path: PrivacyPolicyPath) -> Markup {
    templates::page::legal::privacy_policy()
}
