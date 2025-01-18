use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users").fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, first_name, date_created, date_deleted) VALUES (?, ?, ?, ?)")
        .bind(&user.id)
        .bind(&user.first_name)
        .bind(&user.date_created)
        .bind(&user.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET first_name = ?, date_deleted = ? WHERE id = ?")
        .bind(&user.first_name)
        .bind(&user.date_deleted)
        .bind(&user.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE ID = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map(|_| ())
}
