use secrecy::SecretString;
use crate::{model::user::{self, User, UserId}, value::{DateTime, PasswordHash}};

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
        handle: "admin".to_string(),
        password_hash: PasswordHash::from_plain_password(plain_password.clone()),
        date_created: DateTime::now(),
        date_deleted: None,
    };
    user::create(pool, &user).await.unwrap();

    (user, plain_password)
}
