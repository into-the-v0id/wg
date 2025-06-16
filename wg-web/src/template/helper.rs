use chrono::{Datelike, Days};
use icu_calendar::Gregorian;
use icu_datetime::{fieldsets, FixedCalendarDateTimeFormatter};
use icu_locale_core::Locale;
use wg_core::value::Date;
use crate::{Translations, LANGUAGE, TRANSLATIONS};

pub fn t() -> Translations {
    TRANSLATIONS.get()
}

fn date_to_icu_date(date: Date) -> icu_datetime::input::Date<Gregorian> {
    let chrono_date: chrono::NaiveDate = date.into();

    let year_data = chrono_date.year_ce();
    let mut year = year_data.1 as i32;
    if ! year_data.0 {
        year *= -1;
    }

    icu_datetime::input::Date::try_new_gregorian(
        year,
        (chrono_date.month0() + 1) as u8,
        (chrono_date.day0() + 1) as u8,
    ).unwrap()
}

pub fn format_date_long(date: Date) -> String {
    let langauge = LANGUAGE.get();
    let locale: Locale = langauge.to_string().parse().unwrap();

    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.into(),
        fieldsets::YMD::long(),
    ).unwrap();

    let icu_date = date_to_icu_date(date);

    formatter.format(&icu_date).to_string()
}

pub fn format_date_long_simple(date: Date) -> String {
    let chrono_date: chrono::NaiveDate = date.into();
    let chrono_now: chrono::NaiveDate = Date::now().into();
    let chrono_yesterday = chrono_now.checked_sub_days(Days::new(1)).unwrap();
    let chrono_tomorrow = chrono_now.checked_add_days(Days::new(1)).unwrap();

    if chrono_date == chrono_now {
        t().today().to_string()
    } else if chrono_date == chrono_yesterday {
        t().yesterday().to_string()
    } else if chrono_date == chrono_tomorrow {
        t().tomorrow().to_string()
    } else if chrono_date.iso_week() == chrono_now.iso_week() {
        let langauge = LANGUAGE.get();
        let locale: Locale = langauge.to_string().parse().unwrap();

        let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
            locale.into(),
            fieldsets::E::long(),
        ).unwrap();

        let icu_date = date_to_icu_date(date);

        formatter.format(&icu_date).to_string()
    } else if chrono_date.year_ce() == chrono_now.year_ce() {
        let langauge = LANGUAGE.get();
        let locale: Locale = langauge.to_string().parse().unwrap();

        let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
            locale.into(),
            fieldsets::MD::long(),
        ).unwrap();

        let icu_date = date_to_icu_date(date);

        formatter.format(&icu_date).to_string()
    } else {
        format_date_long(date)
    }
}
