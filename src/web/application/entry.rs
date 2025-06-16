use crate::model::authentication_session::AuthenticationSession;
use axum::response::Redirect;

use super::{authentication::LoginPath, chore_list::ChoreListIndexPath};

pub async fn redirect(auth_session: Option<AuthenticationSession>) -> Redirect {
    if auth_session.is_some() {
        Redirect::to(ChoreListIndexPath.to_string().as_str())
    } else {
        Redirect::to(LoginPath.to_string().as_str())
    }
}
