use fluent_static::MessageBundle;
use lettre::{message::{header::ContentType, Mailbox}, Message};
use maud::html;
use wg_core::model::{chore::Chore, chore_list::ChoreList, user::User};
use crate::{layout, message_builder, Translations, DEFAULT_LANGAGE};

pub fn low_score_reminder(
    user: &User,
    chore_lists: &[&ChoreList],
    due_chores: &[&Chore],
) -> Message {
    let language = user.last_used_language.unwrap_or(DEFAULT_LANGAGE);
    let t = Translations::get(&language.to_string()).unwrap();

    let html = layout::default(
        &language,
        &t.message_low_score_reminder_title(),
        html! {
            p { (t.greeting(&user.name)) }

            p { (t.message_low_score_reminder_content_lists(chore_lists.len())) }

            ul {
                @for chore_list in chore_lists.iter() {
                    li { (chore_list.name) }
                }
            }

            @if !due_chores.is_empty() {
                p { (t.message_low_score_reminder_content_due_chores(due_chores.len())) }

                ul {
                    @for due_chore in due_chores.iter() {
                        li { (due_chore.name) }
                    }
                }
            }
        },
    ).into_string();

    message_builder()
        .to(Mailbox::new(Some(user.name.clone()), user.email.parse().unwrap()))
        .subject(t.message_low_score_reminder_title().to_string())
        .header(ContentType::TEXT_HTML)
        .body(html)
        .unwrap()
}
