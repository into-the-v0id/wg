
use super::value::{Date, DateTime, Uuid};

#[derive(Debug, sqlx::FromRow)]
pub struct ChoreActivity {
    pub id: Uuid,
    pub chore_id: Uuid,
    pub user_id: Uuid,
    pub date: Date,
    pub comment: Option<String>,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl ChoreActivity {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<ChoreActivity, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_activities WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<ChoreActivity>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_activitie ORDER BY date DESC, date_created DESC").fetch_all(pool).await
}

pub async fn get_all_for_chore(pool: &sqlx::sqlite::SqlitePool, chore_id: &Uuid) -> Result<Vec<ChoreActivity>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_activities WHERE chore_id = ? ORDER BY date DESC, date_created DESC").bind(chore_id).fetch_all(pool).await
}

pub async fn get_all_for_chore_list(pool: &sqlx::sqlite::SqlitePool, chore_list_id: &Uuid) -> Result<Vec<ChoreActivity>, sqlx::Error> {
    sqlx::query_as("
        SELECT chore_activities.* FROM chore_activities
        INNER JOIN chores ON chore_activities.chore_id = chores.id
        WHERE chores.chore_list_id = ?
        ORDER BY date DESC, date_created DESC
    ")
        .bind(chore_list_id)
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore_activity: &ChoreActivity) -> Result<(), sqlx::Error> {
    tracing::info!(chore_activity = ?chore_activity, "Creating chore activity");

    sqlx::query("INSERT INTO chore_activities (id, chore_id, user_id, date, comment, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&chore_activity.id)
        .bind(&chore_activity.chore_id)
        .bind(&chore_activity.user_id)
        .bind(&chore_activity.date)
        .bind(&chore_activity.comment)
        .bind(&chore_activity.date_created)
        .bind(&chore_activity.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore_activity: &ChoreActivity) -> Result<(), sqlx::Error> {
    tracing::info!(chore_activity = ?chore_activity, "Updating chore activity");

    sqlx::query("UPDATE chore_activities SET chore_id = ?, user_id = ?, date = ?, comment = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore_activity.chore_id)
        .bind(&chore_activity.user_id)
        .bind(&chore_activity.date)
        .bind(&chore_activity.comment)
        .bind(&chore_activity.date_deleted)
        .bind(&chore_activity.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, chore_activity: &ChoreActivity) -> Result<(), sqlx::Error> {
    tracing::info!(chore_activity = ?chore_activity, "Deleting chore activity");

    sqlx::query("DELETE FROM chore_activities WHERE ID = ?")
        .bind(&chore_activity.id)
        .execute(pool)
        .await
        .map(|_| ())
}
