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

use lettre::{message::{header::ContentType, Mailbox}, Message};
use wg_core::model::user::User;
use crate::message_builder;

pub fn low_score_reminder(user: &User) -> Message {
    message_builder()
        .to(Mailbox::new(Some(user.name.clone()), "user@local.local".parse().unwrap()))
        .subject("Test Subject")
        .header(ContentType::TEXT_PLAIN)
        .body("Test Body".to_string())
        .unwrap()
}
