use fluent_static::MessageBundle;
use lettre::{message::{header::ContentType, Mailbox}, Message};
use maud::html;
use wg_core::model::user::User;
use crate::{layout, message_builder, Translations};

pub fn low_score_reminder(user: &User) -> Message {
    let t = Translations::get("en").unwrap();

    let html = layout::default(
        &t.message_low_score_reminder_title(),
        html! {
            "Hi " (user.name) ","
            br;br;
            (t.message_low_score_reminder_content())
        },
    ).into_string();

    message_builder()
        .to(Mailbox::new(Some(user.name.clone()), "user@local.local".parse().unwrap()))
        .subject(t.message_low_score_reminder_title().to_string())
        .header(ContentType::TEXT_HTML)
        .body(html)
        .unwrap()
}
