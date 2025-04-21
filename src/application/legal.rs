use maud::Markup;
use crate::templates;

pub async fn view_privacy_policy() -> Markup {
    templates::page::legal::privacy_policy()
}
