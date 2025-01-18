use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM users").fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users VALUES (?, ?)")
        .bind(&user.id)
        .bind(&user.first_name)
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
