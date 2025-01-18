use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ChoreActivity {
    pub id: Uuid,
    pub chore_id: Uuid,
    pub user_id: Uuid,
    pub date: chrono::NaiveDate,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
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
    sqlx::query_as("SELECT * FROM chore_activitie ORDER BY date DESC").fetch_all(pool).await
}

pub async fn get_all_for_chore(pool: &sqlx::sqlite::SqlitePool, chore_id: &Uuid) -> Result<Vec<ChoreActivity>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_activities WHERE chore_id = ? ORDER BY date DESC").bind(chore_id).fetch_all(pool).await
}

pub async fn get_all_for_chore_list(pool: &sqlx::sqlite::SqlitePool, chore_list_id: &Uuid) -> Result<Vec<ChoreActivity>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_activities WHERE chore_id IN (SELECT id FROM chores WHERE chore_list_id = ?) ORDER BY date DESC")
        .bind(chore_list_id)
        .fetch_all(pool)
        .await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore_activity: &ChoreActivity) -> Result<(), sqlx::Error> {
    tracing::info!("Created chore activity with ID {}", chore_activity.id);

    sqlx::query("INSERT INTO chore_activities (id, chore_id, user_id, date, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&chore_activity.id)
        .bind(&chore_activity.chore_id)
        .bind(&chore_activity.user_id)
        .bind(&chore_activity.date)
        .bind(&chore_activity.date_created)
        .bind(&chore_activity.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore_activity: &ChoreActivity) -> Result<(), sqlx::Error> {
    tracing::info!("Updated chore activity with ID {}", chore_activity.id);

    sqlx::query("UPDATE chore_activities SET chore_id = ?, user_id = ?, date = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore_activity.chore_id)
        .bind(&chore_activity.user_id)
        .bind(&chore_activity.date)
        .bind(&chore_activity.date_deleted)
        .bind(&chore_activity.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    tracing::info!("Deleted chore activity with ID {}", id);

    sqlx::query("DELETE FROM chore_activities WHERE ID = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
}
