pub use sqlx;
use sqlx::migrate::MigrateDatabase;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

pub type Pool = sqlx::sqlite::SqlitePool;

pub async fn create_pool(path: String) -> Pool {
    let db_url = format!("sqlite:{}", path);

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        tracing::info!("Creating database {}", &db_url);
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    sqlx::sqlite::SqlitePool::connect(&db_url).await.unwrap()
}
