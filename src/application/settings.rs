use maud::Markup;
use crate::{domain::authentication_session::AuthenticationSession, templates};

pub async fn view(auth_session: AuthenticationSession) -> Markup {
    templates::page::settings::settings(auth_session)
}
