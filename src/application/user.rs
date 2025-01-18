use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;
use crate::domain::user;

#[derive(Template)]
#[template(path = "user/list.jinja")]
struct ListTemplate {
    users: Vec<user::User>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let users = user::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {users}.render().unwrap())
}

#[derive(Template)]
#[template(path = "user/detail.jinja")]
struct DetailTemplate {
    user: user::User,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let user = user::get_by_id(&state.pool, &id).await.unwrap(); // TODO: respond with 404 if not found

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
    let user = user::User {
        id: Uuid::now_v7(),
        first_name: payload.first_name,
        date_created: chrono::offset::Utc::now(),
        date_deleted: None,
    };

    // TODO: gracefully handle error
    user::create(&state.pool, &user).await.unwrap();

    Redirect::to(&format!("/users/{}", user.id))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let mut user = user::get_by_id(&state.pool, &id).await.unwrap(); // TODO: respond with 404 if not found
    assert!(!user.is_deleted());

    user.date_deleted = Some(chrono::offset::Utc::now());

    user::update(&state.pool, &user).await.unwrap();

    Redirect::to("/users")
}

pub async fn restore(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let mut user = user::get_by_id(&state.pool, &id).await.unwrap(); // TODO: respond with 404 if not found
    assert!(user.is_deleted());

    user.date_deleted = None;

    user::update(&state.pool, &user).await.unwrap();

    Redirect::to("/users")
}
