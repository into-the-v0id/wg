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

pub mod message;
mod layout;

use std::io;
use fluent_static::message_bundle;
use lettre::{address::Envelope, message::{Mailbox, MessageBuilder}, AsyncSendmailTransport, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
pub use lettre;

#[message_bundle(
    resources = [
        ("translations/en.ftl", "en"),
        ("translations/de.ftl", "de"),
    ],
    default_language = "en",
)]
pub struct Translations;

pub enum MailTransport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Sendmail(AsyncSendmailTransport<Tokio1Executor>),
}

#[async_trait::async_trait]
impl AsyncTransport for MailTransport {
    type Ok = ();
    type Error = io::Error;

    async fn send_raw(&self, envelope: &Envelope, email: &[u8]) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Smtp(transport) => transport.send_raw(envelope, email).await
                .map(|_| ())
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err)),
            Self::Sendmail(transport) => transport.send_raw(envelope, email).await
                .map(|_| ())
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err)),
        }
    }

    async fn shutdown(&self) {
        match self {
            Self::Smtp(transport) => transport.shutdown().await,
            Self::Sendmail(transport) => transport.shutdown().await,
        }
    }
}

pub fn message_builder() -> MessageBuilder {
    let mut builder = Message::builder();

    let raw_from = std::env::var("MAIL_FROM").unwrap();
    let mut from = raw_from.parse::<Mailbox>().unwrap();
    if from.name.is_none() {
        from.name = Some("WG".to_string());
    }
    builder = builder.from(from);

    if let Ok(raw_reply_to) = std::env::var("MAIL_REPLY_TO") {
        let reply_to = raw_reply_to.parse::<Mailbox>().unwrap();
        builder = builder.reply_to(reply_to);
    }

    builder
}
