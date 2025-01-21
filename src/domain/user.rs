use uuid::fmt::Hyphenated as HyphenatedUuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: HyphenatedUuid,
    pub name: String,
    pub handle: String,
    pub password_hash: String,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_deleted: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &HyphenatedUuid) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_by_handle(pool: &sqlx::sqlite::SqlitePool, handle: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE handle = ?").bind(handle).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users").fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!("Created {:?}", user);

    sqlx::query("INSERT INTO users (id, name, handle, password_hash, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.handle)
        .bind(&user.password_hash)
        .bind(&user.date_created)
        .bind(&user.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!("Updated {:?}", user);

    sqlx::query("UPDATE users SET name = ?, handle = ?, password_hash = ?, date_deleted = ? WHERE id = ?")
        .bind(&user.name)
        .bind(&user.handle)
        .bind(&user.password_hash)
        .bind(&user.date_deleted)
        .bind(&user.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    tracing::info!("Deleted {:?}", user);

    sqlx::query("DELETE FROM users WHERE ID = ?")
        .bind(&user.id)
        .execute(pool)
        .await
        .map(|_| ())
}
