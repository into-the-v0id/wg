use std::sync::Arc;
use futures::FutureExt;
use wg_mail::lettre::AsyncTransport;
use wg_core::model::{self, chore::Chore, chore_list::ChoreList};
use crate::AppState;

pub async fn low_score_reminder(state: Arc<AppState>) {
    let (all_users, all_chore_lists, all_chores, low_score_users) = tokio::try_join!(
        model::user::get_all(&state.pool),
        model::chore_list::get_all(&state.pool),
        model::chore::get_all(&state.pool),
        wg_core::service::user::get_low_score_users(&state.pool).map(|r| Ok(r)),
    ).unwrap();

    for (user_id, chore_list_ids) in low_score_users.iter() {
        let user = all_users.iter().find(|u| &u.id == user_id).unwrap();
        if user.is_deleted() {
            continue;
        }

        let chore_lists = all_chore_lists.iter()
            .filter(|cl| chore_list_ids.contains(&cl.id))
            .collect::<Vec<&ChoreList>>();

        let due_chores = all_chores.iter()
            .filter(|c| chore_list_ids.contains(&c.chore_list_id) && c.is_due().unwrap_or(false))
            .collect::<Vec<&Chore>>();

        let mail_message = wg_mail::message::low_score_reminder(user, &chore_lists, &due_chores);
        state.mail_transport.send(mail_message).await.unwrap();
    }
}
