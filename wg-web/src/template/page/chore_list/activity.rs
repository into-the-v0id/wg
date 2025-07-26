use maud::{html, Markup};
use crate::handler::chore::ChoreDetailPath;
use crate::handler::chore_activity::ChoreActivityCreatePath;
use crate::handler::chore_activity::ChoreActivityDeletePath;
use crate::handler::chore_activity::ChoreActivityDetailPath;
use crate::handler::chore_activity::ChoreActivityIndexPath;
use crate::handler::chore_activity::ChoreActivityRestorePath;
use crate::handler::chore_activity::ChoreActivityUpdatePath;
use crate::handler::chore_list::ChoreListIndexPath;
use crate::handler::chore_list_user::ChoreListUserDetailPath;
use wg_core::model::authentication_session::AuthenticationSession;
use wg_core::model::chore_list;
use wg_core::model::chore_activity;
use wg_core::model::chore;
use wg_core::model::user;
use wg_core::value::Date;
use wg_core::value::DateTime;
use crate::template::helper::format_date_long;
use crate::template::helper::format_date_long_simple;
use crate::template::helper::t;
use crate::template::layout;
use crate::template::partial;
use crate::template::partial::navigation::ChoreListNavigationItem;

pub fn list(
    chore_list: chore_list::ChoreList,
    activities_by_date: Vec<(Date, Vec<&chore_activity::ChoreActivity>)>,
    deleted_activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
    users: Vec<user::User>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().activities())
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreListIndexPath.to_string().as_str())
            .meta_actions(html! {
                @if !chore_list.is_deleted() {
                    a.secondary.subtle href=(ChoreActivityCreatePath { chore_list_id: chore_list.id }) { "+ " (t().add_action()) }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
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
                            @let chore = chores.iter().find(|chore| chore.id == activity.chore_id).unwrap();
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (t().points_value_short(chore.points))

                                        " – " (user.name)

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

            @if ! deleted_activities.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { (t().deleted_activities()) }
                    ul.card-container.collapse {
                        @for activity in deleted_activities {
                            @let chore = chores.iter().find(|chore| chore.id == activity.chore_id).unwrap();
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (t().points_value_short(chore.points))

                                        " – " (user.name)

                                        " – " time datetime=(activity.date.format("%Y-%m-%d")) title=(activity.date.format("%Y-%m-%d")) {
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

pub fn detail(
    activity: chore_activity::ChoreActivity,
    chore: chore::Chore,
    chore_list: chore_list::ChoreList,
    user: user::User,
    auth_session: AuthenticationSession,
    allow_edit: bool,
    allow_delete_restore: bool,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().activity())
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreActivityIndexPath { chore_list_id: chore_list.id }.to_string().as_str())
            .meta_actions(html! {
                @if activity.is_deleted() {
                    @if allow_delete_restore {
                        button.link.secondary.subtle.mb-0 type="submit" form="activity_restore" { "↻ " (t().restore_action()) }
                        form #activity_restore method="post" action=(ChoreActivityRestorePath {chore_list_id:chore_list.id, chore_activity_id: activity.id }) { }
                    }
                } @else if !chore.is_deleted() && !chore_list.is_deleted() && activity.user_id == auth_session.user_id {
                    @if allow_delete_restore {
                        button.link.secondary.subtle.mb-0 type="submit" form="activity_delete" { "✗ " (t().delete_action()) }
                    }

                    @if allow_edit {
                        a.secondary.subtle href=(ChoreActivityUpdatePath {chore_list_id:chore_list.id, chore_activity_id: activity.id }) style="margin-left: 1.25rem;" { "✎ " (t().edit_action()) }
                    }

                    @if allow_delete_restore {
                        form #activity_delete method="post" action=(ChoreActivityDeletePath {chore_list_id:chore_list.id, chore_activity_id: activity.id }) { }
                    }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            @if activity.is_deleted() || chore.is_deleted() || chore_list.is_deleted() {
                div {
                    em { (t().activity_has_been_deleted()) }
                }

                br;
            }

            dl {
                dt { (t().date()) }
                dd {
                    time datetime=(activity.date.format("%Y-%m-%d")) title=(activity.date.format("%Y-%m-%d")) {
                        (format_date_long(activity.date))
                    }
                }

                dt { (t().user()) }
                dd { a.inherit.subtle href=(ChoreListUserDetailPath { chore_list_id: chore_list.id, user_id: user.id }) { "👤 " (user.name) } }

                dt { (t().chore()) }
                dd {
                    a.inherit.subtle href=(ChoreDetailPath { chore_list_id: chore_list.id, chore_id: chore.id }) {
                        "🧹 " (chore.name) " (" (chore.points) "P)"
                    }
                }

                @if let Some(comment) = activity.comment {
                    dt { (t().comment()) }
                    dd { (comment) }
                }
            }
        },
    )
}

pub fn create(
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    min_date: Date,
    max_date: Date,
    now: DateTime,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().create_activity())
            .back_url(ChoreActivityIndexPath { chore_list_id: chore_list.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { (t().chore()) }
                select #chore_id name="chore_id" required {
                    option selected disabled hidden value="" { }
                    @for chore in chores {
                        @if !chore.is_deleted() {
                            option value=(chore.id) { (chore.name) }
                        }
                    }
                }

                label for="date" { (t().date()) }
                input #date name="date" type="date" min=(min_date.format("%Y-%m-%d")) max=(max_date.format("%Y-%m-%d")) value=(now.format("%Y-%m-%d")) required;

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
    activity: chore_activity::ChoreActivity,
    chores: Vec<chore::Chore>,
    chore_list: chore_list::ChoreList,
    min_date: Date,
    max_date: Date,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().edit_activity())
            .back_url(ChoreActivityDetailPath { chore_list_id: chore_list.id, chore_activity_id: activity.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { (t().chore()) }
                select #chore_id name="chore_id" required {
                    option disabled hidden value="" { }
                    @for chore in chores {
                        @if !chore.is_deleted() {
                            option value=(chore.id) selected[chore.id == activity.chore_id] { (chore.name) }
                        }
                    }
                }

                label for="date" { (t().date()) }
                input #date name="date" type="date" min=(min_date.format("%Y-%m-%d")) max=(max_date.format("%Y-%m-%d")) value=(activity.date.format("%Y-%m-%d")) required;

                label for="comment" {
                    (t().comment())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #comment name="comment" {
                    @if let Some(comment) = activity.comment {
                        (comment)
                    }
                }

                button type="submit" { (t().save_action()) }
            }
        },
    )
}
