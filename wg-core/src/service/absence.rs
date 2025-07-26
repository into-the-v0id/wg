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

pub fn get_group_date(absence: &Absence) -> Date {
    if absence.is_in_past() {
        absence.date_end.unwrap()
    } else if absence.is_in_future() {
        absence.date_start
    } else {
        Date::now()
    }
}
