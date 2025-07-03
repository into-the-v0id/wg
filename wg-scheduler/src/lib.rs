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

mod job;

use std::{str::FromStr, sync::Arc};

use lettre::{AsyncSendmailTransport, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use tracing::Instrument;
use chrono::Utc;
use cron::Schedule;
use tokio::time::Instant;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub enum MailTransport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Sendmail(AsyncSendmailTransport<Tokio1Executor>),
}

impl MailTransport {
    async fn send(&self, message: lettre::Message) {
        match self {
            Self::Smtp(transport) => { transport.send(message).await.unwrap(); },
            Self::Sendmail(transport) => { transport.send(message).await.unwrap(); },
        };
    }
}

pub struct AppState {
    pub pool: wg_core::db::Pool,
    pub mail_transport: MailTransport,
}

pub async fn start(state: AppState, cancel_token: CancellationToken) {
    let state = Arc::new(state);

    let tracker = TaskTracker::new();

    if let Ok(cron) = std::env::var("LOW_SCORE_REMINDER_CRON") {
        tracker.spawn(start_cron(
            "low_score_reminder",
            Schedule::from_str(&cron).unwrap(),
            job::low_score_reminder,
            state.clone(),
            cancel_token.clone(),
        ));
    }

    tracker.close();

    tracing::debug!("Scheduler started");

    tracker.wait().await;

    tracing::debug!("Scheduler stopped");
}

async fn start_cron<F, R, S>(
    name: &str,
    schedule: Schedule,
    job: F,
    state: S,
    cancel_token: CancellationToken
) where
    F: Fn(S) -> R + Send + Sync + 'static,
    R: Future<Output = ()> + Send,
    S: Clone + Send + 'static,
{
    tracing::debug!(name = name, schedule = %schedule, "Registered cron job");

    let job = Arc::new(job);

    while let Some(next_date) = schedule.upcoming(Utc).next() {
        tracing::debug!(name = name, next_run = %next_date, "Next run scheduled");

        let next_instant = Instant::now() + next_date.signed_duration_since(Utc::now()).to_std().unwrap();

        tokio::select! {
            _ = tokio::time::sleep_until(next_instant) => {},
            _ = cancel_token.cancelled() => {
                break;
            },
        };

        let span = tracing::error_span!(
            "cron_job",
            name = name,
        );

        tracing::debug!(name = name, "Running cron job");

        let job_clone = job.clone();
        let state_clone = state.clone();

        let job_result = tokio::spawn(async move {
            job_clone(state_clone).instrument(span).await;
        }).await;

        if let Err(err) = job_result {
            if let Ok(panic) = err.try_into_panic() {
                if let Some(s) = panic.downcast_ref::<String>() {
                    tracing::error!("Cron job panicked: {}", s);
                } else if let Some(s) = panic.downcast_ref::<&str>() {
                    tracing::error!("Cron job panicked: {}", s);
                } else {
                    tracing::error!("Cron job panicked but unable to downcast the panic info");
                }
            }
        } else {
            tracing::debug!(name = name, "Finished running cron job");
        }
    }
}
