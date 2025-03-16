use chrono::Datelike;
use super::value::{DateTime, Uuid};

#[derive(Debug, Copy, Clone, PartialEq, strum::EnumString, strum::Display, strum::AsRefStr, strum::IntoStaticStr, strum::EnumIter, serde::Serialize, serde::Deserialize, sqlx::Type)]
pub enum ScoreResetInterval {
    Monthly,
    Quaterly,
    HalfYearly,
    Yearly,
    Never,
}

impl ScoreResetInterval {
    pub fn as_months(&self) -> Option<u32> {
        match *self {
            ScoreResetInterval::Monthly => Some(1),
            ScoreResetInterval::Quaterly => Some(3),
            ScoreResetInterval::HalfYearly => Some(6),
            ScoreResetInterval::Yearly => Some(12),
            ScoreResetInterval::Never => None,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ChoreList {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub score_reset_interval: ScoreResetInterval,
    pub date_created: DateTime,
    pub date_deleted: Option<DateTime>,
}

impl ChoreList {
    pub fn is_deleted(&self) -> bool {
        self.date_deleted.is_some()
    }
}

pub async fn get_by_id(pool: &sqlx::sqlite::SqlitePool, id: &Uuid) -> Result<ChoreList, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_lists WHERE id = ?").bind(id).fetch_one(pool).await
}

pub async fn get_all(pool: &sqlx::sqlite::SqlitePool) -> Result<Vec<ChoreList>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM chore_lists").fetch_all(pool).await
}

pub async fn get_score_per_user(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<Vec<(Uuid, i32)>, sqlx::Error> {
    let mut interval_start_date = chrono::NaiveDate::MIN;
    let mut interval_end_date = chrono::NaiveDate::MAX;

    if let Some(interval_duration_months) = chore_list.score_reset_interval.as_months() {
        let now = chrono::offset::Utc::now();
        let current_month = now.month();

        let elapsed_months = (current_month - 1) % interval_duration_months;

        let interval_start_month = current_month - elapsed_months;

        interval_start_date = chrono::NaiveDate::from_ymd_opt(now.year(), interval_start_month, 1).unwrap();
        interval_end_date = interval_start_date
            .checked_add_months(chrono::Months::new(interval_duration_months)).unwrap()
            .checked_sub_days(chrono::Days::new(1)).unwrap();
    }

    sqlx::query_as::<_, (Uuid, i32)>("
        SELECT users.id as user_id, SUM(chores.points) as total_score FROM chore_activities
        INNER JOIN chores ON chore_activities.chore_id = chores.id
        INNER JOIN chore_lists ON chores.chore_list_id = chore_lists.id
        INNER JOIN users ON chore_activities.user_id = users.id
        WHERE chore_lists.id = ?
            AND chore_activities.date_deleted IS NULL AND chores.date_deleted IS NULL AND chore_lists.date_deleted IS NULL AND users.date_deleted IS NULL
            AND chore_activities.date >= ? AND chore_activities.date <= ?
        GROUP BY users.id
        ORDER BY total_score DESC
    ").bind(chore_list.id).bind(interval_start_date).bind(interval_end_date).fetch_all(pool).await.map(|r| r.into_iter().collect())
}

pub async fn create(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!(chore_list = ?chore_list, "Creating chore list");

    sqlx::query("INSERT INTO chore_lists (id, name, description, score_reset_interval, date_created, date_deleted) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&chore_list.id)
        .bind(&chore_list.name)
        .bind(&chore_list.description)
        .bind(&chore_list.score_reset_interval)
        .bind(&chore_list.date_created)
        .bind(&chore_list.date_deleted)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn update(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!(chore_list = ?chore_list, "Updating chore list");

    sqlx::query("UPDATE chore_lists SET name = ?, description = ?, score_reset_interval = ?, date_deleted = ? WHERE id = ?")
        .bind(&chore_list.name)
        .bind(&chore_list.description)
        .bind(&chore_list.score_reset_interval)
        .bind(&chore_list.date_deleted)
        .bind(&chore_list.id)
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn delete(pool: &sqlx::sqlite::SqlitePool, chore_list: &ChoreList) -> Result<(), sqlx::Error> {
    tracing::info!(chore_list = ?chore_list, "Deleting chore list");

    sqlx::query("DELETE FROM chore_lists WHERE ID = ?")
        .bind(&chore_list.id)
        .execute(pool)
        .await
        .map(|_| ())
}
