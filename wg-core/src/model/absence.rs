use crate::model::user::UserId;
use crate::value::{Date, Tagged};
use crate::value::{DateTime, Uuid};

pub type AbsenceId = Tagged<Uuid, Absence>;

#[derive(Debug, sqlx::FromRow)]
pub struct Absence {
    pub id: AbsenceId,
    pub user_id: UserId,
    pub date_start: Date,
    pub date_end: Option<Date>,
    pub comment: Option<String>,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl Absence {
    pub fn is_in_past(&self) -> bool {
        match self.date_end {
            Some(date_end) => date_end.is_in_past(),
            None => false,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.date_end {
            Some(date_end) => self.date_start.is_in_past_or_today() && date_end.is_in_future_or_today(),
            None => self.date_start.is_in_past_or_today(),
        }
    }

    pub fn is_in_future(&self) -> bool {
        self.date_start.is_in_future()
    }

    pub fn num_days(&self) -> Option<u32> {
        self.date_end.map(|date_end| date_end.as_ref().signed_duration_since(*self.date_start.as_ref()).num_days() as u32)
    }

    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &AbsenceId) -> Result<Absence, sqlx::Error> {
    sqlx::query_as("SELECT * FROM absences WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Absence>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM absences ORDER BY date_end DESC NULLS FIRST, date_start DESC, date_created DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_active(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Absence>, sqlx::Error> {
    let now = Date::now();

    sqlx::query_as("SELECT * FROM absences WHERE date_start <= ? AND (date_end IS NULL OR date_end >= ?) ORDER BY date_end DESC NULLS FIRST, date_start DESC, date_created DESC")
        .bind(now)
        .bind(now)
        .fetch_all(pool)
        .await
}

pub async fn get_active_in_period(pool: &sqlx::sqlite::SqlitePool, start_date: Date, end_date: Date) -> Result<Vec<Absence>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM absences WHERE date_start <= ? AND (date_end IS NULL OR date_end >= ?) ORDER BY date_end DESC NULLS FIRST, date_start DESC, date_created DESC")
        .bind(end_date)
        .bind(start_date)
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, absence: &Absence) -> Result<(), sqlx::Error> {
    tracing::info!(absence = ?absence, "Creating absence");

    sqlx::query("INSERT INTO absences (id, user_id, date_start, date_end, comment, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(absence.id)
        .bind(absence.user_id)
        .bind(absence.date_start)
        .bind(absence.date_end)
        .bind(&absence.comment)
        .bind(absence.date_created)
        .bind(absence.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, absence: &Absence) -> Result<(), sqlx::Error> {
    tracing::info!(absence = ?absence, "Updating absence");

    sqlx::query("UPDATE absences SET user_id = ?, date_start = ?, date_end = ?, comment = ?, date_deleted = ? WHERE id = ?")
        .bind(absence.user_id)
        .bind(absence.date_start)
        .bind(absence.date_end)
        .bind(&absence.comment)
        .bind(absence.date_deleted)
        .bind(absence.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, absence: &Absence) -> Result<(), sqlx::Error> {
    tracing::info!(absence = ?absence, "Deleting absence");

    sqlx::query("DELETE FROM absences WHERE id = ?")
        .bind(absence.id)
        .execute(pool)
        .await
        .map(|_| ())
}
