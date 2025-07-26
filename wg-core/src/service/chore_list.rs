use crate::{model::{self, absence::Absence, chore_list::ChoreList, user::UserId}, service, value::Date};

pub async fn get_adjusted_score_per_user(
    pool: &crate::db::Pool,
    chore_list: &ChoreList,
) -> Result<Vec<(UserId, i32)>, sqlx::Error> {
    let (interval_start_date, _interval_end_date) = if let Some(interval_start_and_end_date) = chore_list.score_reset_interval.get_current_start_and_end_date() {
        interval_start_and_end_date
    } else {
        match model::chore_activity::get_oldest_not_deleted_for_chore_list(pool, &chore_list.id).await {
            Ok(oldest_activity) => (oldest_activity.date, Date::now()),
            Err(sqlx::Error::RowNotFound) => (Date::now(), Date::now()),
            Err(err) => return Err(err),
        }
    };

    let interval_passed_days = Date::now().as_ref().signed_duration_since(*interval_start_date.as_ref()).num_days() + 1;

    let (score_per_user, absences) = tokio::try_join!(
        model::chore_list::get_score_per_user(pool, chore_list),
        model::absence::get_active_in_period(pool, interval_start_date, Date::now())
    ).unwrap();

    let adjusted_score_per_user = score_per_user.iter()
        .map(|&(user_id, score)| {
            let user_absences = absences.iter()
                .filter(|absence| absence.user_id == user_id)
                .collect::<Vec<&Absence>>();

            let absent_num_days = service::absence::count_num_days_in_period(
                user_absences,
                Some(interval_start_date),
                Some(Date::now()),
            ) as i64;
            let present_num_days = interval_passed_days - absent_num_days;

            let adjusted_score = if score == 0 || absent_num_days == 0 || present_num_days == 0 || interval_passed_days == 0 {
                score
            } else {
                (score as f64 / present_num_days as f64 * interval_passed_days as f64).round() as i32
            };

            (user_id, adjusted_score)
        })
        .collect();

    Ok(adjusted_score_per_user)
}
