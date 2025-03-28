
use super::value::{DateTime, Uuid};

#[derive(Debug, sqlx::FromRow)]
pub struct Chore {
    pub id: Uuid,
    pub chore_list_id: Uuid,
    pub name: String,
    pub points: i32,
    pub description: Option<String>,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl Chore {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<Chore, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores ORDER BY points").fetch_all(pool).await
}

pub async fn get_all_for_chore_list(pool: &sqlx::sqlite::SqlitePool, chore_list_id: &Uuid) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE chore_list_id = ? ORDER BY points").bind(chore_list_id).fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Creating chore");

    sqlx::query("INSERT INTO chores (id, chore_list_id, name, points, description, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&chore.id)
        .bind(&chore.chore_list_id)
        .bind(&chore.name)
        .bind(&chore.points)
        .bind(&chore.description)
        .bind(&chore.date_created)
        .bind(&chore.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Updating chore");

    sqlx::query("UPDATE chores SET chore_list_id = ?, name = ?, points = ?, description = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore.chore_list_id)
        .bind(&chore.name)
        .bind(&chore.points)
        .bind(&chore.description)
        .bind(&chore.date_deleted)
        .bind(&chore.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!(chore = ?chore, "Deleting chore");

    sqlx::query("DELETE FROM chores WHERE ID = ?")
        .bind(&chore.id)
        .execute(pool)
        .await
        .map(|_| ())
}
