use crate::domain::authentication_session::AuthenticationSession;
use axum::response::Redirect;

pub async fn redirect(auth_session: Option<AuthenticationSession>) -> Redirect {
    if auth_session.is_some() {
        Redirect::to("/chore-lists")
    } else {
        Redirect::to("/login")
    }
}
