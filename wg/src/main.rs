// Copyright (C) Oliver Amann
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use secrecy::ExposeSecret;
use wg_core::service;
use sqlx::migrate::MigrateDatabase;
use tracing_subscriber::EnvFilter;
use wg_core::MIGRATOR;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let pool = create_db_pool().await;

    MIGRATOR.run(&pool).await.unwrap();

    if !service::user::exists_any_user(&pool).await {
        let (admin_user, admin_password) = service::user::create_default_admin_user(&pool).await;

        println!(
            "Created user with handle '{}' and password '{}'",
            admin_user.handle, admin_password.expose_secret()
        );
    }

    let web_router = wg_web::make_router(wg_web::AppState {
        pool: pool,
    });

    let port = std::env::var("PORT")
        .map(|raw_port| raw_port.parse::<i32>().unwrap())
        .unwrap_or(3000);
    let address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Listening on http://{} ...", listener.local_addr().unwrap());

    wg_web::start(listener, web_router).await
}

async fn create_db_pool() -> sqlx::sqlite::SqlitePool {
    let db_file = std::env::var("DB_FILE").unwrap_or(String::from("./data/sqlite.db"));
    let db_url = format!("sqlite:{}", db_file);

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        tracing::info!("Creating database {}", &db_url);
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    sqlx::sqlite::SqlitePool::connect(&db_url).await.unwrap()
}
