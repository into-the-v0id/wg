use serde_with::serde_as;
use wg_core::model::absence;
use wg_core::model::absence::AbsenceId;
use wg_core::model::user;
use wg_core::service;
use crate::extractor::authentication::AuthSession;
use crate::extractor::model::Absence;
use crate::template;
use crate::AppState;
use wg_core::value::{Date, DateTime};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use chrono::Days;
use maud::Markup;
use std::sync::Arc;

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences")]
pub struct AbsenceIndexPath;

pub async fn view_list(
    _path: AbsenceIndexPath,
    State(state): State<Arc<AppState>>,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let (users, all_absences) = tokio::try_join!(
        user::get_all(&state.pool),
        absence::get_all(&state.pool),
    ).unwrap();

    let (absences, deleted_absences): (Vec<_>, Vec<_>) = all_absences
        .into_iter()
        .partition(|absence| !absence.is_deleted());
    let (absences, mut future_absences): (Vec<_>, Vec<_>) = absences
        .into_iter()
        .partition(|absence| !absence.is_in_future());
    let absences_by_date = service::absence::group_and_sort_by_date(absences.iter().collect(), true);

    future_absences.sort_by(|a, b| a.date_start.as_ref().cmp(b.date_start.as_ref()).reverse());

    Ok(template::page::absence::list(
        future_absences,
        absences_by_date,
        deleted_absences,
        users,
    ))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences/{absence_id}")]
pub struct AbsenceDetailPath {
    pub absence_id: AbsenceId,
}

pub async fn view_detail(
    _path: AbsenceDetailPath,
    Absence(absence): Absence,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let user = user::get_by_id(&state.pool, &absence.user_id).await.unwrap();

    let allow_edit = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(4)).date_naive(),
        None => true,
    };
    let allow_delete_restore = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(6)).date_naive(),
        None => true,
    };

    Ok(template::page::absence::detail(
        absence,
        user,
        auth_session,
        allow_edit,
        allow_delete_restore,
    ))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences/create")]
pub struct AbsenceCreatePath;

pub async fn view_create_form(
    _path: AbsenceCreatePath,
    AuthSession(_auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    let min_start_date = Date::from((chrono::Utc::now() - Days::new(4)).date_naive());
    let now = DateTime::now();

    Ok(template::page::absence::create(
        min_start_date,
        now,
    ))
}

#[serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct CreatePayload {
    date_start: Date,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    date_end: Option<Date>,
    comment: String,
}

pub async fn create(
    _path: AbsenceCreatePath,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
    Form(payload): Form<CreatePayload>,
) -> Result<Redirect, StatusCode> {
    let min_start_date = (chrono::Utc::now() - Days::new(4)).date_naive();

    if payload.date_start.as_ref() < &min_start_date {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let absence = absence::Absence {
        id: AbsenceId::new(),
        user_id: auth_session.user_id,
        date_start: payload.date_start,
        date_end: payload.date_end,
        comment: match payload.comment.trim() {
            "" => None,
            comment => Some(comment.to_string()),
        },
        date_created: DateTime::now(),
        date_deleted: None,
    };

    absence::create(&state.pool, &absence)
        .await
        .unwrap();

    Ok(Redirect::to(&AbsenceIndexPath.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences/{absence_id}/update")]
pub struct AbsenceUpdatePath {
    pub absence_id: AbsenceId,
}

pub async fn view_update_form(
    _path: AbsenceUpdatePath,
    Absence(absence): Absence,
    AuthSession(auth_session): AuthSession,
) -> Result<Markup, StatusCode> {
    if absence.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if absence.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let min_start_date = Date::from(
        chrono::Utc::now().date_naive()
            .min((chrono::Utc::now() - Days::new(4)).date_naive())
    );

    let allow_edit = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(4)).date_naive(),
        None => true,
    };
    if !allow_edit {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(template::page::absence::update(
        absence,
        min_start_date,
    ))
}

#[serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct UpdatePayload {
    date_start: Date,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    date_end: Option<Date>,
    comment: String,
}

pub async fn update(
    _path: AbsenceUpdatePath,
    Absence(mut absence): Absence,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
    Form(payload): Form<UpdatePayload>,
) -> Result<Redirect, StatusCode> {
    if absence.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if absence.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let allow_edit = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(4)).date_naive(),
        None => true,
    };
    if !allow_edit {
        return Err(StatusCode::FORBIDDEN);
    }

    let min_start_date = chrono::Utc::now().date_naive()
        .min((chrono::Utc::now() - Days::new(4)).date_naive());
    if payload.date_start.as_ref() < &min_start_date {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let is_invlid_order = match payload.date_end {
        Some(date_end) => date_end.as_ref() < payload.date_start.as_ref(),
        None => false,
    };
    if is_invlid_order {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    absence.date_start = payload.date_start;
    absence.date_end = payload.date_end;
    absence.comment = match payload.comment.trim() {
        "" => None,
        comment => Some(comment.to_string()),
    };

    absence::update(&state.pool, &absence)
        .await
        .unwrap();

    Ok(Redirect::to(&AbsenceDetailPath { absence_id: absence.id }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences/{absence_id}/delete")]
pub struct AbsenceDeletePath {
    pub absence_id: AbsenceId,
}

pub async fn delete(
    _path: AbsenceDeletePath,
    Absence(mut absence): Absence,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Redirect, StatusCode> {
    if absence.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if absence.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let allow_delete_restore = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(6)).date_naive(),
        None => true,
    };
    if !allow_delete_restore {
        return Err(StatusCode::FORBIDDEN);
    }

    absence.date_deleted = Some(DateTime::now());

    absence::update(&state.pool, &absence)
        .await
        .unwrap();

    Ok(Redirect::to(&AbsenceDetailPath { absence_id: absence.id }.to_string().as_str()))
}

#[derive(TypedPath, serde::Deserialize)]
#[typed_path("/absences/{absence_id}/restore")]
pub struct AbsenceRestorePath {
    pub absence_id: AbsenceId,
}

pub async fn restore(
    _path: AbsenceRestorePath,
    Absence(mut absence): Absence,
    State(state): State<Arc<AppState>>,
    AuthSession(auth_session): AuthSession,
) -> Result<Redirect, StatusCode> {
    if !absence.is_deleted() {
        return Err(StatusCode::FORBIDDEN);
    }

    if absence.user_id != auth_session.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let allow_delete_restore = match absence.date_end {
        Some(date_end) => date_end.as_ref() >= &(chrono::Utc::now() - Days::new(6)).date_naive(),
        None => true,
    };
    if !allow_delete_restore {
        return Err(StatusCode::FORBIDDEN);
    }

    absence.date_deleted = None;

    absence::update(&state.pool, &absence)
        .await
        .unwrap();

    Ok(Redirect::to(&AbsenceDetailPath { absence_id: absence.id }.to_string().as_str()))
}
