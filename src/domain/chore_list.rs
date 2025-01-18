use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ChoreList {
    pub id: Uuid,
    pub name: String,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl ChoreList {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<ChoreList, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_lists WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<ChoreList>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_lists").fetch_all(pool).await
}

pub async fn get_score_per_user(pool: &sqlx::sqlite::SqlitePool, chore_list_id: &Uuid) -> Result<HashMap<Uuid, i32>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, i32)>("
        SELECT users.id as user_id, SUM(chores.points) as score FROM chore_activities
        INNER JOIN chores ON chore_activities.chore_id = chores.id
        INNER JOIN chore_lists ON chores.chore_list_id = chore_lists.id
        INNER JOIN users ON chore_activities.user_id = users.id
        WHERE chore_lists.id = ?
        GROUP BY users.id
    ").bind(chore_list_id).fetch_all(pool).await.map(|r| r.into_iter().collect())
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!("Created {:?}", chore_list);

    sqlx::query("INSERT INTO chore_lists (id, name, date_created, date_deleted) VALUES (?, ?, ?, ?)")
        .bind(&chore_list.id)
        .bind(&chore_list.name)
        .bind(&chore_list.date_created)
        .bind(&chore_list.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!("Updated {:?}", chore_list);

    sqlx::query("UPDATE chore_lists SET name = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore_list.name)
        .bind(&chore_list.date_deleted)
        .bind(&chore_list.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!("Deleted {:?}", chore_list);

    sqlx::query("DELETE FROM chore_lists WHERE ID = ?")
        .bind(&chore_list.id)
        .execute(pool)
        .await
        .map(|_| ())
}
