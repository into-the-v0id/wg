use maud::{html, Markup};
use crate::handler::absence::AbsenceCreatePath;
use crate::handler::absence::AbsenceDeletePath;
use crate::handler::absence::AbsenceDetailPath;
use crate::handler::absence::AbsenceIndexPath;
use crate::handler::absence::AbsenceRestorePath;
use crate::handler::absence::AbsenceUpdatePath;
use crate::handler::user::UserDetailPath;
use crate::template::partial::navigation::GlobalNavigationItem;
use wg_core::model::authentication_session::AuthenticationSession;
use wg_core::model::absence;
use wg_core::model::user;
use wg_core::value::Date;
use wg_core::value::DateTime;
use crate::template::helper::format_date_long;
use crate::template::helper::format_date_long_simple;
use crate::template::helper::t;
use crate::template::layout;
use crate::template::partial;

pub fn list(
    future_absences: Vec<absence::Absence>,
    absences_by_date: Vec<(Date, Vec<&absence::Absence>)>,
    deleted_absences: Vec<absence::Absence>,
    users: Vec<user::User>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🏖️")
            .title(&t().absences())
            .headline(&format!("🏖️ {}", t().absences()))
            .meta_actions(html! {
                a.secondary.subtle href=(AbsenceCreatePath) { "+ " (t().add_action()) }
            })
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Absences)))
            .build(),
        html! {
            @if ! future_absences.is_empty() {
                details {
                    summary.arrow-left.text-muted { (t().future_absences()) }
                    ul.card-container.collapse {
                        @for absence in future_absences {
                            @let user = users.iter().find(|user| user.id == absence.user_id).unwrap();

                            li {
                                a.card href=(AbsenceDetailPath { absence_id: absence.id }) {
                                    div.title {
                                        (format_date_long_simple(absence.date_start))
                                        " - "
                                        @match absence.date_end {
                                            Some(date_end) => (format_date_long_simple(date_end)),
                                            None => "?",
                                        }
                                    }

                                    small.text-muted {
                                        (user.name)

                                        @if let Some(num_days) = absence.num_days() {
                                            " – " (t().n_days(num_days))
                                        }

                                        @if absence.comment.is_some() {
                                            " – " (t().has_comment())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                br;
            }

            div.timeline {
                @for (date, absences_of_date) in absences_by_date {
                    div.timeline-date-separator {
                        time datetime=(date.format("%Y-%m-%d")) title=(date.format("%Y-%m-%d")) {
                            (format_date_long_simple(date))
                        }
                    }
                    ul.card-container.collapse {
                        @for absence in absences_of_date {
                            @let user = users.iter().find(|user| user.id == absence.user_id).unwrap();

                            li {
                                a.card href=(AbsenceDetailPath { absence_id: absence.id }) {
                                    div.title {
                                        (format_date_long_simple(absence.date_start))
                                        " - "
                                        @match absence.date_end {
                                            Some(date_end) => (format_date_long_simple(date_end)),
                                            None => "?",
                                        }
                                    }

                                    small.text-muted {
                                        (user.name)

                                        @if let Some(num_days) = absence.num_days() {
                                            " – " (t().n_days(num_days))
                                        }

                                        @if absence.comment.is_some() {
                                            " – " (t().has_comment())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if ! deleted_absences.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { (t().deleted_absences()) }
                    ul.card-container.collapse {
                        @for absence in deleted_absences {
                            @let user = users.iter().find(|user| user.id == absence.user_id).unwrap();

                            li {
                                a.card href=(AbsenceDetailPath { absence_id: absence.id }) {
                                    div.title {
                                        (format_date_long_simple(absence.date_start))
                                        " - "
                                        @match absence.date_end {
                                            Some(date_end) => (format_date_long_simple(date_end)),
                                            None => "?",
                                        }
                                    }

                                    small.text-muted {
                                        (user.name)

                                        @if let Some(num_days) = absence.num_days() {
                                            " – " (t().n_days(num_days))
                                        }

                                        @if absence.comment.is_some() {
                                            " – " (t().has_comment())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn detail(
    absence: absence::Absence,
    user: user::User,
    auth_session: AuthenticationSession,
    allow_edit: bool,
    allow_delete_restore: bool,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🏖️")
            .title(&t().absence())
            .headline(&format!("🏖️ {}", t().absence()))
            .back_url(AbsenceIndexPath.to_string().as_str())
            .meta_actions(html! {
                @if absence.is_deleted() {
                    @if allow_delete_restore {
                        button.link.secondary.subtle.mb-0 type="submit" form="absence_restore" { "↻ " (t().restore_action()) }
                        form #absence_restore method="post" action=(AbsenceRestorePath { absence_id: absence.id }) { }
                    }
                } @else if absence.user_id == auth_session.user_id {
                    @if allow_delete_restore {
                        button.link.secondary.subtle.mb-0 type="submit" form="absence_delete" { "✗ " (t().delete_action()) }
                    }

                    @if allow_edit {
                        a.secondary.subtle href=(AbsenceUpdatePath { absence_id: absence.id }) style="margin-left: 1.25rem;" { "✎ " (t().edit_action()) }
                    }

                    @if allow_delete_restore {
                        form #absence_delete method="post" action=(AbsenceDeletePath { absence_id: absence.id }) { }
                    }
                }
            })
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Absences)))
            .build(),
        html! {
            @if absence.is_deleted() {
                div {
                    em { (t().absence_has_been_deleted()) }
                }

                br;
            }

            dl {
                dt { (t().absence_start_date()) }
                dd {
                    time datetime=(absence.date_start.format("%Y-%m-%d")) title=(absence.date_start.format("%Y-%m-%d")) {
                        (format_date_long(absence.date_start))
                    }
                }

                dt { (t().absence_end_date()) }
                dd {
                    @if let Some(date_end) = absence.date_end {
                        time datetime=(date_end.format("%Y-%m-%d")) title=(date_end.format("%Y-%m-%d")) {
                            (format_date_long(date_end))
                        }
                    } @else {
                        (t().unknown())
                    }
                }

                @if let Some(num_days) = absence.num_days() {
                    dt { (t().duration()) }
                    dd { (t().n_days(num_days)) }
                }

                dt { (t().user()) }
                dd { a.inherit.subtle href=(UserDetailPath { user_id: user.id }) { "👤 " (user.name) } }

                @if let Some(comment) = absence.comment {
                    dt { (t().comment()) }
                    dd { (comment) }
                }
            }
        },
    )
}

pub fn create(
    min_date: Date,
    now: DateTime,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🏖️")
            .title(&t().create_absence())
            .headline(&format!("🏖️ {}", t().create_absence()))
            .back_url(AbsenceIndexPath.to_string().as_str())
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Absences)))
            .build(),
        html! {
            form method="post" {
                label for="date_start" { (t().absence_start_date()) }
                input #date_start name="date_start" type="date" min=(min_date.format("%Y-%m-%d")) value=(now.format("%Y-%m-%d")) required;

                label for="date_end" {
                    (t().absence_end_date())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                input #date_end name="date_end" type="date" min=(min_date.format("%Y-%m-%d"));

                label for="comment" {
                    (t().comment())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #comment name="comment" { }

                button type="submit" { (t().create_action()) }
            }
        },
    )
}

pub fn update(
    absence: absence::Absence,
    min_start_date: Date,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🏖️")
            .title(&t().edit_absence())
            .headline(&format!("🏖️ {}", t().edit_absence()))
            .back_url(AbsenceDetailPath { absence_id: absence.id }.to_string().as_str())
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Absences)))
            .build(),
        html! {
            form method="post" {
                label for="date_start" { (t().absence_start_date()) }
                input #date_start name="date_start" type="date" min=(min_start_date.format("%Y-%m-%d")) value=(absence.date_start.format("%Y-%m-%d")) required;

                label for="date_end" {
                    (t().absence_end_date())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                input #date_end name="date_end" type="date" min=(min_start_date.format("%Y-%m-%d")) value=[absence.date_end.map(|date| date.format("%Y-%m-%d"))];

                label for="comment" {
                    (t().comment())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #comment name="comment" {
                    @if let Some(comment) = absence.comment {
                        (comment)
                    }
                }

                button type="submit" { (t().save_action()) }
            }
        },
    )
}
