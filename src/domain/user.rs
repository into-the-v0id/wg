use super::value::{DateTime, PasswordHash, Uuid};

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub handle: String,
    pub password_hash: PasswordHash,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl User {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_by_handle(
    pool: &sqlx::sqlite::SqlitePool,
    handle: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE handle = ?")
        .bind(handle)
        .fetch_one(pool)
        .await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users").fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!(user = ?user, "Creating user");

    sqlx::query("INSERT INTO users (id, name, handle, password_hash, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(user.id)
        .bind(&user.name)
        .bind(&user.handle)
        .bind(&user.password_hash)
        .bind(user.date_created)
        .bind(user.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!(user = ?user, "Updating user");

    sqlx::query("UPDATE users SET name = ?, handle = ?, password_hash = ?, date_deleted = ? WHERE id = ?")
        .bind(&user.name)
        .bind(&user.handle)
        .bind(&user.password_hash)
        .bind(user.date_deleted)
        .bind(user.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!(user = ?user, "Deleting user");

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user.id)
        .execute(pool)
        .await
        .map(|_| ())
}
