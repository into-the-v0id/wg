use std::sync::Arc;
use futures::FutureExt;
use lettre::{message::{header::ContentType, Mailbox}, Message};
use wg_core::model::{self, chore_list::ChoreList};
use crate::AppState;

pub async fn low_score_reminder(state: Arc<AppState>) {
    let (all_users, all_chore_lists, low_score_users) = tokio::try_join!(
        model::user::get_all(&state.pool),
        model::chore_list::get_all(&state.pool),
        wg_core::service::user::get_low_score_users(&state.pool).map(|r| Ok(r)),
    ).unwrap();

    for (user_id, chore_list_ids) in low_score_users.iter() {
        let user = all_users.iter().find(|u| &u.id == user_id).unwrap();
        let _chore_lists = all_chore_lists.iter().filter(|cl| chore_list_ids.contains(&cl.id)).collect::<Vec<&ChoreList>>();

        let mail_message = Message::builder()
            .from(Mailbox::new(Some("Local".to_string()), "test@local.local".parse().unwrap()))
            // .reply_to(Mailbox::new(Some("Local".to_string()), "test@local.local".parse().unwrap()))
            .to(Mailbox::new(Some("Oliver Amann".to_string()), "user@local.local".parse().unwrap()))
            .subject("Test Subject")
            .header(ContentType::TEXT_PLAIN)
            .body("Test Body".to_string())
            .unwrap();

        state.mail_transport.send(mail_message).await;

        println!("Send mail to {}", user.name);
    }
}
