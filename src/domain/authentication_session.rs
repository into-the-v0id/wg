use crate::domain::value::Tagged;

use super::{user::UserId, value::{DateTime, Uuid}};

pub type AuthenticationSessionId = Tagged<Uuid, AuthenticationSession>;

#[derive(Clone, sqlx::FromRow)]
pub struct AuthenticationSession {
    pub id: AuthenticationSessionId,
    pub token: String,
    pub user_id: UserId,
    pub date_expires: DateTime,
    pub date_created: DateTime,
}

impl AuthenticationSession {
    pub fn is_expired(&self) -> bool {
        self.date_expires.as_ref() < DateTime::now().as_ref()
    }
}

pub async fn get_by_id(
    pool: &sqlx::sqlite::SqlitePool,
    id: &AuthenticationSessionId,
) -> Result<AuthenticationSession, sqlx::Error> {
    sqlx::query_as("SELECT * FROM authentication_sessions WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_by_token(
    pool: &sqlx::sqlite::SqlitePool,
    token: &str,
) -> Result<AuthenticationSession, sqlx::Error> {
    sqlx::query_as("SELECT * FROM authentication_sessions WHERE token = ?")
        .bind(token)
        .fetch_one(pool)
        .await
}

pub async fn get_all(
    pool: &sqlx::sqlite::SqlitePool,
) -> Result<Vec<AuthenticationSession>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM authentication_sessions")
        .fetch_all(pool)
        .await
}

pub async fn get_all_for_user(
    pool: &sqlx::sqlite::SqlitePool,
    user_id: &UserId,
) -> Result<Vec<AuthenticationSession>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM authentication_sessions WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn create(
    pool: &sqlx::sqlite::SqlitePool,
    auth_session: &AuthenticationSession,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO authentication_sessions (id, token, user_id, date_expires, date_created) VALUES (?, ?, ?, ?, ?)")
        .bind(auth_session.id)
        .bind(&auth_session.token)
        .bind(auth_session.user_id)
        .bind(auth_session.date_expires)
        .bind(auth_session.date_created)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(
    pool: &sqlx::sqlite::SqlitePool,
    auth_session: &AuthenticationSession,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE authentication_sessions SET token = ?, user_id = ?, date_expires = ? WHERE id = ?")
        .bind(&auth_session.token)
        .bind(auth_session.user_id)
        .bind(auth_session.date_expires)
        .bind(auth_session.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(
    pool: &sqlx::sqlite::SqlitePool,
    auth_session: &AuthenticationSession,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM authentication_sessions WHERE id = ?")
        .bind(auth_session.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete_all_expired(pool: &sqlx::sqlite::SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM authentication_sessions WHERE date_expires < ?")
        .bind(DateTime::now())
        .execute(pool)
        .await
        .map(|_| ())
}
