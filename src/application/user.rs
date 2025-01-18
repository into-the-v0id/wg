use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;

#[derive(Template)]
#[template(path = "user/list.jinja")]
struct ListTemplate {
    users: Vec<crate::domain::user::User>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let users = crate::domain::user::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {users}.render().unwrap())
}

#[derive(Template)]
#[template(path = "user/detail.jinja")]
struct DetailTemplate {
    user: crate::domain::user::User,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let user = crate::domain::user::get_by_id(&state.pool, &id).await.unwrap();

    Html(DetailTemplate {user}.render().unwrap())
}

#[derive(Template)]
#[template(path = "user/create.jinja")]
struct CreateTemplate();

pub async fn view_create_form() -> Html<String> {
    Html(CreateTemplate().render().unwrap())
}

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreatePayload {
    first_name: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreatePayload>,
) -> Redirect {
    let user = crate::domain::user::User {
        id: Uuid::now_v7(),
        first_name: payload.first_name,
    };

    crate::domain::user::create(&state.pool, &user).await.unwrap();

    Redirect::to(&format!("/users/{}", user.id))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let user = crate::domain::user::get_by_id(&state.pool, &id).await.unwrap();

    crate::domain::user::delete(&state.pool, &user.id).await.unwrap();

    Redirect::to("/users")
}
