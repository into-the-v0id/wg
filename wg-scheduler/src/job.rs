use std::sync::Arc;
use futures::FutureExt;
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

        println!("Send mail to {}", user.name);
    }
}
