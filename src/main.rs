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

pub mod model;
pub mod value;
pub mod web;

use model::user::UserId;
use value::{DateTime, PasswordHash};
use sqlx::migrate::MigrateDatabase;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let pool = create_db_pool().await;

    sqlx::migrate!().run(&pool).await.unwrap();

    create_user_if_necessary(&pool).await;

    let web_router = web::make_router(web::AppState {
        pool: pool,
    });

    let port = std::env::var("PORT")
        .map(|raw_port| raw_port.parse::<i32>().unwrap())
        .unwrap_or(3000);
    let address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Listening on http://{} ...", listener.local_addr().unwrap());

    axum::serve(listener, web_router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
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

/// If no users exist, create a user and print ist credentials.
/// Mainly used for new-installs without an existing DB.
async fn create_user_if_necessary(pool: &sqlx::sqlite::SqlitePool) {
    let users = model::user::get_all(pool).await.unwrap();
    if !users.is_empty() {
        return;
    }

    let mut plain_password_buf = [0u8; 8];
    getrandom::getrandom(&mut plain_password_buf).unwrap();
    let plain_password = const_hex::encode(plain_password_buf);

    let user = model::user::User {
        id: UserId::new(),
        name: "Admin".to_string(),
        handle: "admin".to_string(),
        password_hash: PasswordHash::from_plain_password(plain_password.clone().into()),
        date_created: DateTime::now(),
        date_deleted: None,
    };
    model::user::create(pool, &user).await.unwrap();

    println!(
        "Created user with handle '{}' and password '{}'",
        user.handle, plain_password
    );
}
