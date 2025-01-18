use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub first_name: String,
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<Person, sqlx::Error> {
    sqlx::query_as("SELECT * FROM people WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<Person>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM people").fetch_all(pool).await
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, person: &Person) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO people VALUES (?, ?)")
        .bind(&person.id)
        .bind(&person.first_name)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM people WHERE ID = ?")
        .bind(&id)
        .execute(pool)
        .await
        .map(|_| ())
}
