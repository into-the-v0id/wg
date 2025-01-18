use std::sync::Arc;
use askama::Template;
use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use uuid::Uuid;
use crate::AppState;

#[derive(Template)]
#[template(path = "person/list.jinja")]
struct ListTemplate {
    people: Vec<crate::domain::person::Person>,
}

pub async fn view_list(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let people = crate::domain::person::get_all(&state.pool).await.unwrap();

    Html(ListTemplate {people}.render().unwrap())
}

#[derive(Template)]
#[template(path = "person/detail.jinja")]
struct DetailTemplate {
    person: crate::domain::person::Person,
}

pub async fn view_detail(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let person = crate::domain::person::get_by_id(&state.pool, &id).await.unwrap();

    Html(DetailTemplate {person}.render().unwrap())
}

#[derive(Template)]
#[template(path = "person/create.jinja")]
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
    let person = crate::domain::person::Person {
        id: Uuid::now_v7(),
        first_name: payload.first_name,
    };

    crate::domain::person::create(&state.pool, &person).await.unwrap();

    Redirect::to(&format!("/people/{}", person.id))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let person = crate::domain::person::get_by_id(&state.pool, &id).await.unwrap();

    crate::domain::person::delete(&state.pool, &person.id).await.unwrap();

    Redirect::to("/people")
}
