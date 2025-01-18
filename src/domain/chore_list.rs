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

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
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
    sqlx::query("UPDATE chore_lists SET name = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore_list.name)
        .bind(&chore_list.date_deleted)
        .bind(&chore_list.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chore_lists WHERE ID = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map(|_| ())
}
