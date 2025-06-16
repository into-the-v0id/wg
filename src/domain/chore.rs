use chrono::Days;

use crate::domain::chore_list::ChoreListId;
use crate::domain::value::Tagged;
use super::chore_activity;
use super::value::{Date, DateTime, Uuid};

pub type ChoreId = Tagged<Uuid, Chore>;

#[derive(Debug, sqlx::FromRow)]
pub struct Chore {
    pub id: ChoreId,
    pub chore_list_id: ChoreListId,
    pub name: String,
    pub points: u32,
    pub interval_days: Option<u32>,
    pub next_due_date: Option<Date>,
    pub description: Option<String>,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl Chore {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &ChoreId) -> Result<Chore, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores ORDER BY points")
        .fetch_all(pool)
        .await
}

pub async fn get_all_for_chore_list(
    pool: &sqlx::sqlite::SqlitePool,
    chore_list_id: &ChoreListId,
) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE chore_list_id = ? ORDER BY points")
        .bind(chore_list_id)
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Creating chore");

    sqlx::query("INSERT INTO chores (id, chore_list_id, name, points, interval_days, next_due_date, description, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(chore.id)
        .bind(chore.chore_list_id)
        .bind(&chore.name)
        .bind(chore.points)
        .bind(chore.interval_days)
        .bind(chore.next_due_date)
        .bind(&chore.description)
        .bind(chore.date_created)
        .bind(chore.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Updating chore");

    sqlx::query("UPDATE chores SET chore_list_id = ?, name = ?, points = ?, interval_days = ?, next_due_date = ?, description = ?, date_deleted = ? WHERE id = ?")
        .bind(chore.chore_list_id)
        .bind(&chore.name)
        .bind(chore.points)
        .bind(chore.interval_days)
        .bind(chore.next_due_date)
        .bind(&chore.description)
        .bind(chore.date_deleted)
        .bind(chore.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Deleting chore");

    sqlx::query("DELETE FROM chores WHERE id = ?")
        .bind(chore.id)
        .execute(pool)
        .await
        .map(|_| ())
}

/// Returns true if changes were made and false if nothing changed
pub async fn update_next_due_date(
    chore: &mut Chore,
    pool: &sqlx::sqlite::SqlitePool,
    save_to_db: bool,
) -> Result<bool, sqlx::Error> {
    if let Some(interval_days) = chore.interval_days {
        let last_activity_date =
            match chore_activity::get_latest_not_deleted_for_chore(pool, &chore.id).await {
                Ok(chore_activity) => chore_activity.date,
                Err(sqlx::Error::RowNotFound) => {
                    Date::from(chore.date_created.as_ref().date_naive())
                }
                Err(err) => return Err(err),
            };

        let next_due_date = Some(Date::from(
            last_activity_date.as_ref().clone() + Days::new(interval_days.into()),
        ));

        if chore.next_due_date != next_due_date {
            chore.next_due_date = next_due_date;

            if save_to_db {
                update(pool, chore).await?;
            }

            return Ok(true);
        }

        Ok(false)
    } else {
        if chore.next_due_date.is_some() {
            chore.next_due_date = None;

            if save_to_db {
                update(pool, chore).await?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}
