use chrono::Days;
use crate::{model::absence::Absence, value::Date};

pub fn group_and_sort_by_date(
    mut absences: Vec<&Absence>,
    sort_latest_first: bool,
) -> Vec<(Date, Vec<&Absence>)> {
    absences.sort_by(|a, b| {
        get_group_date(a).cmp(&get_group_date(b))
            .then_with(|| a.date_start.cmp(&b.date_start))
            .then_with(|| a.date_created.cmp(&b.date_created))
    });
    if sort_latest_first {
        absences.reverse();
    }

    let mut absences_by_date = Vec::new();

    let mut current_date: Option<Date> = None;
    let mut current_absences: Vec<&Absence> = Vec::new();
    for absence in absences.iter() {
        let group_date = get_group_date(absence);

        if current_date.is_none() {
            current_date = Some(group_date);
        }

        if current_date.unwrap() != group_date {
            if !current_absences.is_empty() {
                absences_by_date.push((current_date.unwrap(), current_absences));
                current_absences = Vec::new();
            }

            current_date = Some(group_date);
        }

        current_absences.push(absence);
    }

    if !current_absences.is_empty() {
        absences_by_date.push((current_date.unwrap(), current_absences));
    }

    absences_by_date
}

fn get_group_date(absence: &Absence) -> Date {
    if absence.is_in_past() {
        absence.date_end.unwrap()
    } else if absence.is_in_future() {
        absence.date_start
    } else {
        Date::now()
    }
}

/// Counts the number of absent days in the specified period, making sure to count overlapping absences only once
pub fn count_num_days_in_period(mut absences: Vec<&Absence>, period_date_start: Option<Date>, period_date_end: Option<Date>) -> u32 {
    absences.sort_by(|a, b| a.date_start.cmp(&b.date_start));

    let mut num_absent_days = 0;
    let mut processed_until_date = period_date_start.map(|date| Date::from(date.as_ref().clone() - Days::new(1)));
    for absence in absences.iter() {
        if let Some(period_date_end) = period_date_end && absence.date_start > period_date_end {
            continue;
        }

        let date_start = if let Some(processed_until_date) = processed_until_date && absence.date_start <= processed_until_date {
            Date::from(processed_until_date.as_ref().clone() + Days::new(1))
        } else {
            absence.date_start
        };

        let mut date_end = absence.date_end.unwrap_or_else(|| Date::now());
        if let Some(period_date_end) = period_date_end && date_end > period_date_end {
            date_end = period_date_end;
        }

        if date_end <= date_start {
            continue;
        }

        num_absent_days += date_end.as_ref().signed_duration_since(*date_start.as_ref()).num_days() as u32;
        processed_until_date = Some(date_end);
    }

    num_absent_days
}
