use std::collections::HashMap;
use secrecy::SecretString;
use crate::{model::{chore_list::{self, ChoreListId}, user::{self, User, UserId}}, value::{DateTime, PasswordHash}};

pub async fn exists_any_user(pool: &crate::db::Pool) -> bool {
    let users = user::get_all(pool).await.unwrap();

    !users.is_empty()
}

// Returns the created user and the plain password
pub async fn create_default_admin_user(pool: &crate::db::Pool) -> (User, SecretString) {
    let mut plain_password_buf = [0u8; 8];
    getrandom::getrandom(&mut plain_password_buf).unwrap();
    let plain_password: SecretString = const_hex::encode(plain_password_buf).into();

    let user = User {
        id: UserId::new(),
        name: "Admin".to_string(),
        email: "admin@localhost".to_string(),
        password_hash: PasswordHash::from_plain_password(plain_password.clone()),
        last_used_language: None,
        date_created: DateTime::now(),
        date_deleted: None,
    };
    user::create(pool, &user).await.unwrap();

    (user, plain_password)
}

pub async fn get_low_score_users(pool: &crate::db::Pool) -> HashMap<UserId, Vec<ChoreListId>> {
    let mut low_score_users = HashMap::new();

    let chore_lists = chore_list::get_all(pool).await.unwrap();
    for chore_list in chore_lists.into_iter() {
        if chore_list.is_deleted() {
            continue;
        }

        let score_per_user = chore_list::get_score_per_user(pool, &chore_list).await.unwrap();
        let scores = score_per_user.iter().map(|(_, score)| score).cloned().collect::<Vec<i32>>();

        let min_score = match scores.iter().min() {
            Some (min) => min,
            None => continue,
        };
        let max_score = match scores.iter().max() {
            Some (max) => max,
            None => continue,
        };

        let score_delta = max_score - min_score;
        let score_threshold = min_score + (score_delta as f32 * 0.25).ceil() as i32;

        let current_low_score_users = score_per_user.iter()
            .filter(|(_, score)| score <= &score_threshold)
            .map(|(user_id, _)| user_id)
            .cloned()
            .collect::<Vec<UserId>>();

        for user_id in current_low_score_users {
            low_score_users.entry(user_id)
                .and_modify(|user_chore_lists: &mut Vec<ChoreListId>| {
                    if !user_chore_lists.contains(&chore_list.id) {
                        user_chore_lists.push(chore_list.id);
                    }
                })
                .or_insert_with(|| Vec::from([chore_list.id]));
        }
    }

    low_score_users
}
