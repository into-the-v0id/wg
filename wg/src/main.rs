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

use lettre::{AsyncSendmailTransport, AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use wg_core::{db::Pool, service};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let db_file = std::env::var("DB_FILE")
        .unwrap_or(String::from("./data/sqlite.db"));
    let pool = wg_core::db::create_pool(db_file).await;

    wg_core::db::MIGRATOR.run(&pool).await.unwrap();

    if !service::user::exists_any_user(&pool).await {
        let (admin_user, admin_password) = service::user::create_default_admin_user(&pool).await;

        println!(
            "Created user with email '{}' and password '{}'",
            admin_user.email, admin_password.expose_secret()
        );
    }

    let cancel_token = CancellationToken::new();

    let cancel_token_shutdown = cancel_token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;

        tracing::debug!("Abort signal recieved. Shutting down.");

        cancel_token_shutdown.cancel();
    });

    let tracker = TaskTracker::new();

    tracker.spawn(start_web_server(pool.clone(), cancel_token.clone()));
    tracker.spawn(start_scheduler(pool.clone(), cancel_token.clone()));

    tracker.close();

    tracker.wait().await;
}

async fn start_web_server(pool: Pool, cancel_token: CancellationToken) -> () {
    tracing::debug!("Starting web server");

    let web_router = wg_web::make_router(wg_web::AppState {
        pool: pool,
    });

    let port = std::env::var("PORT")
        .map(|raw_port| raw_port.parse::<i32>().unwrap())
        .unwrap_or(3000);
    let address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("Listening on http://{} ...", listener.local_addr().unwrap());

    wg_web::start(listener, web_router, cancel_token).await
}

async fn start_scheduler(pool: Pool, cancel_token: CancellationToken) -> () {
    tracing::debug!("Starting scheduler");

    let mail_transport = if let Ok(url) = std::env::var("SMTP_URL") {
        wg_mail::MailTransport::Smtp(AsyncSmtpTransport::<Tokio1Executor>::from_url(&url).unwrap().build())
    } else if let Ok(command) = std::env::var("SENDMAIL_COMMAND") {
        wg_mail::MailTransport::Sendmail(AsyncSendmailTransport::<Tokio1Executor>::new_with_command(command))
    } else {
        wg_mail::MailTransport::Sendmail(AsyncSendmailTransport::<Tokio1Executor>::new())
    };

    let state = wg_scheduler::AppState {
        pool: pool,
        mail_transport,
    };

    wg_scheduler::start(state, cancel_token).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
