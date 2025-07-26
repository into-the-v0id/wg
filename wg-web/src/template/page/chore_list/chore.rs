use maud::{html, Markup};
use crate::handler::chore::ChoreActivitiesPath;
use crate::handler::chore::ChoreCreatePath;
use crate::handler::chore::ChoreDeletePath;
use crate::handler::chore::ChoreDetailPath;
use crate::handler::chore::ChoreIndexPath;
use crate::handler::chore::ChoreRestorePath;
use crate::handler::chore::ChoreUpdatePath;
use crate::handler::chore_activity::ChoreActivityCreatePath;
use crate::handler::chore_activity::ChoreActivityDetailPath;
use crate::handler::chore_list::ChoreListIndexPath;
use wg_core::model::chore_list;
use wg_core::model::chore_activity;
use wg_core::model::chore;
use wg_core::model::user;
use wg_core::value::Date;
use crate::template::helper::format_date_long;
use crate::template::helper::format_date_long_simple;
use crate::template::helper::t;
use crate::template::layout;
use crate::template::partial;
use crate::template::partial::navigation::ChoreListNavigationItem;

pub fn list(
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    deleted_chores: Vec<chore::Chore>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🧹")
            .title(&t().chores())
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreListIndexPath.to_string().as_str())
            .meta_actions(html! {
                @if !chore_list.is_deleted() {
                    a.secondary.subtle href=(ChoreCreatePath { chore_list_id: chore_list.id }) { "+ " (t().add_action()) }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            ul.card-container.collapse {
                @for chore in chores {
                    li {
                        a.card href={ (ChoreDetailPath { chore_list_id: chore_list.id, chore_id: chore.id }) } {
                            div.title { (chore.name) }
                            small.text-muted {
                                (t().points_value_short(chore.points))

                                @if let Some(next_due_date) = chore.next_due_date {
                                    @if next_due_date.is_in_past_or_today() {
                                        " – "
                                        span.text-danger.fw-bold { (t().due_hint()) }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if ! deleted_chores.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { (t().deleted_chores()) }
                    ul.card-container.collapse {
                        @for chore in deleted_chores {
                            li {
                                a.card href=(ChoreDetailPath { chore_list_id: chore_list.id, chore_id: chore.id }) {
                                    div.title { (chore.name) }
                                    small.text-muted { (t().points_value_short(chore.points)) }
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
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🧹")
            .title(&chore.name)
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreIndexPath { chore_list_id: chore_list.id }.to_string().as_str())
            .meta_actions(html! {
                @if chore.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="chore_restore" { "↻ " (t().restore_action()) }
                    form #chore_restore method="post" action=(ChoreRestorePath { chore_list_id: chore_list.id, chore_id: chore.id }) { }
                } @else if !chore_list.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="chore_delete" { "✗ " (t().delete_action()) }

                    a.secondary.subtle href=(ChoreUpdatePath { chore_list_id: chore_list.id, chore_id: chore.id }) style="margin-left: 1.25rem;" { "✎ " (t().edit_action()) }

                    form #chore_delete method="post" action=(ChoreDeletePath { chore_list_id: chore_list.id, chore_id: chore.id }) { }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            @if chore.is_deleted() || chore_list.is_deleted() {
                div {
                    em { (t().chore_has_been_deleted()) }
                }

                br;
            }

            dl {
                dt { (t().points()) }
                dd { (chore.points) }

                @if let Some(interval_days) = chore.interval_days {
                    dt { (t().interval()) }
                    dd { (t().every_n_days(interval_days)) }
                }

                @if let Some(next_due_date) = chore.next_due_date {
                    @let is_due = next_due_date.is_in_past_or_today();
                    dt { (t().next_due_date()) }
                    dd.text-danger[is_due].fw-bold[is_due] {
                        time datetime=(next_due_date.format("%Y-%m-%d")) title=(next_due_date.format("%Y-%m-%d")) {
                            (format_date_long(next_due_date))
                        }
                    }
                }

                @if let Some(description) = chore.description {
                    dt { (t().description()) }
                    dd { (description) }
                }
            }

            br;
            br;

            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    li {
                        a.card href=(ChoreActivitiesPath { chore_list_id: chore_list.id, chore_id: chore.id }) {
                            div.title { "✅ " (t().activities()) }
                        }
                    }
                }
            }
        },
    )
}

pub fn create(
    chore_list: chore_list::ChoreList,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🧹")
            .title(&t().create_chore())
            .back_url(ChoreIndexPath { chore_list_id: chore_list.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            form method="post" {
                label for="name" { (t().name()) }
                input #name name="name" type="text" required;

                label for="description" {
                    (t().description())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #description name="description" { }

                label for="points" { (t().points()) }
                input #points name="points" type="number" min="1" step="1" required;

                label for="interval_days" {
                    (t().interval())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                div role="group" {
                    input #interval_days name="interval_days" type="number" min="1" step="1" aria-describedby="interval_days-help-text";
                    label for="interval_days" { (t().days()) }
                }
                small #interval_days-help-text { (t().chore_interval_help_text()) }

                button type="submit" { (t().create_action()) }
            }
        },
    )
}

pub fn update(
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🧹")
            .title(&t().edit_chore())
            .back_url(ChoreDetailPath { chore_list_id: chore_list.id, chore_id: chore.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            form method="post" {
                label for="name" { (t().name()) }
                input #name name="name" type="text" required value=(chore.name);

                label for="description" {
                    (t().description())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #description name="description" {
                    @if let Some(description) = chore.description {
                        (description)
                    }
                }

                label for="points" { (t().points()) }
                input #points name="points" type="number" min="1" step="1" required value=(chore.points);

                label for="interval_days" {
                    (t().interval())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                div role="group" {
                    input #interval_days name="interval_days" type="number" min="1" step="1" aria-describedby="interval_days-help-text" value=[chore.interval_days];
                    label for="interval_days" { (t().days()) }
                }
                small #interval_days-help-text { (t().chore_interval_help_text()) }

                button type="submit" { (t().save_action()) }
            }
        },
    )
}

pub fn list_activities(
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    activities_by_date: Vec<(Date, Vec<&chore_activity::ChoreActivity>)>,
    deleted_activities: Vec<chore_activity::ChoreActivity>,
    users: Vec<user::User>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().activities())
            .teaser(&t().of_x(format!("🧹 {}", chore.name)))
            .back_url(ChoreDetailPath { chore_list_id: chore_list.id, chore_id: chore.id }.to_string().as_str())
            .meta_actions(html! {
                a.secondary.subtle href=(ChoreActivityCreatePath {chore_list_id:chore_list.id }) { "+ " (t().add_action()) }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            div.timeline {
                @for (date, activities_of_date) in activities_by_date {
                    div.timeline-date-separator {
                        time datetime=(date.format("%Y-%m-%d")) title=(date.format("%Y-%m-%d")) {
                            (format_date_long_simple(date))
                        }
                    }
                    ul.card-container.collapse {
                        @for activity in activities_of_date {
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (user.name) }

                                    @if activity.comment.is_some() {
                                        small.text-muted { (t().has_comment()) }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if ! deleted_activities.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { (t().deleted_activities()) }
                    ul.card-container.collapse {
                        @for activity in deleted_activities {
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (user.name) }

                                    small.text-muted {
                                        time datetime=(activity.date.format("%Y-%m-%d")) title=(activity.date.format("%Y-%m-%d")) {
                                            (format_date_long(activity.date))
                                        }

                                        @if activity.comment.is_some() {
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
