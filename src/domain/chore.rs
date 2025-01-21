use uuid::fmt::Hyphenated as HyphenatedUuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Chore {
    pub id: HyphenatedUuid,
    pub chore_list_id: HyphenatedUuid,
    pub name: String,
    pub points: i32,
    pub description: Option<String>,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl Chore {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &HyphenatedUuid) -> Result<Chore, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores").fetch_all(pool).await
}

pub async fn get_all_for_chore_list(pool: &sqlx::sqlite::SqlitePool, chore_list_id: &HyphenatedUuid) -> Result<Vec<Chore>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chores WHERE chore_list_id = ?").bind(chore_list_id).fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore: &Chore) -> Result<(), sqlx::Error> {
    tracing::info!("Created {:?}", chore);

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
    tracing::info!("Updated {:?}", chore);

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
    tracing::info!("Deleted {:?}", chore);

    sqlx::query("DELETE FROM chores WHERE ID = ?")
        .bind(&chore.id)
        .execute(pool)
        .await
        .map(|_| ())
}
