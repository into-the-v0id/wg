use chrono::Days;
use crate::{model::{self, chore::Chore, chore_activity}, value::Date};

/// Returns true if changes were made and false if nothing changed
pub async fn update_next_due_date(
    chore: &mut Chore,
    pool: &sqlx::sqlite::SqlitePool,
    save_to_db: bool,
) -> Result<bool, sqlx::Error> {
    if let Some(interval_days) = chore.interval_days {
        let last_activity_date =
            match chore_activity::get_latest_not_deleted_for_chore(pool, &chore.id).await {
                Ok(chore_activity) => chore_activity.date,
                Err(sqlx::Error::RowNotFound) => {
                    Date::from(chore.date_created.as_ref().date_naive())
                }
                Err(err) => return Err(err),
            };

        let next_due_date = Some(Date::from(
            last_activity_date.as_ref().clone() + Days::new(interval_days.into()),
        ));

        if chore.next_due_date != next_due_date {
            chore.next_due_date = next_due_date;

            if save_to_db {
                model::chore::update(pool, chore).await?;
            }

            return Ok(true);
        }

        Ok(false)
    } else {
        if chore.next_due_date.is_some() {
            chore.next_due_date = None;

            if save_to_db {
                model::chore::update(pool, chore).await?;
            }

            return Ok(true);
        }

        Ok(false)
    }
}
