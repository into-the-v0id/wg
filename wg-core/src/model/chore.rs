use crate::model::chore_list::ChoreListId;
use crate::value::Tagged;
use crate::value::{Date, DateTime, Uuid};

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
    pub fn is_due(&self) -> Option<bool> {
        self.next_due_date.map(|date| date.is_in_past_or_today())
    }

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

pub async fn get_all_due(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE next_due_date IS NOT NULL AND next_due_date <= ? ORDER BY points")
        .bind(Date::now())
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
