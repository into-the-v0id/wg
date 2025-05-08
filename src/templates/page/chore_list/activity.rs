use maud::{html, Markup};
use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::user;
use crate::domain::value::Date;
use crate::domain::value::DateTime;
use crate::templates::helper::t;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::ChoreListNavigationItem;

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
            .headline(&format!("✅ {}", t().activities()))
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url("/chore-lists")
            .meta_actions(html! {
                @if !chore_list.is_deleted() {
                    a.secondary.subtle href={ "/chore-lists/" (chore_list.id) "/activities/create" } { "+ " (t().add_action()) }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            div.timeline {
                @for (date, activities_of_date) in activities_by_date {
                    div.timeline-date-separator { (date.format("%Y-%m-%d")) }
                    ul.card-container.collapse {
                        @for activity in activities_of_date {
                            @let chore = chores.iter().find(|chore| chore.id == activity.chore_id).unwrap();
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
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
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (t().points_value_short(chore.points))

                                        " – " (user.name)

                                        " – " (activity.date.format("%Y-%m-%d"))

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
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title(&t().activity())
            .headline(&format!("✅ {}", t().activity()))
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(&format!("/chore-lists/{}/activities", chore_list.id))
            .meta_actions(html! {
                @if activity.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="activity_restore" { "↻ " (t().restore_action()) }
                    form #activity_restore method="post" action={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) "/restore" } { }
                } @else if !chore.is_deleted() && !chore_list.is_deleted() && activity.user_id == auth_session.user_id {
                    button.link.secondary.subtle.mb-0 type="submit" form="activity_delete" { "✗ " (t().delete_action()) }

                    @if allow_edit {
                        a.secondary.subtle href="/chore-lists/{{ chore_list.id }}/activities/{{ activity.id }}/update" style="margin-left: 1.25rem;" { "✎ " (t().edit_action()) }
                    }

                    form #activity_delete method="post" action="/chore-lists/{{ chore_list.id }}/activities/{{ activity.id }}/delete" { }
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
                dd { (activity.date.format("%Y-%m-%d")) }

                dt { (t().user()) }
                dd { a.inherit.subtle href={ "/chore-lists/" (chore_list.id) "/users/" (user.id) } { "👤 " (user.name) } }

                dt { (t().chore()) }
                dd {
                    a.inherit.subtle href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) } {
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
            .headline(&format!("✅ {}", t().create_activity()))
            .back_url(&format!("/chore-lists/{}/activities", chore_list.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { (t().chore()) }
                select #chore_id name="chore_id" required {
                    option selected disabled hidden value="" { }
                    @for chore in chores {
                        @if !chore.is_deleted() {
                            option value=(chore.id) {
                                (chore.name)
                                " (" (chore.points) "P)"
                            }
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
            .headline(&format!("✅ {}", t().edit_activity()))
            .back_url(&format!("/chore-lists/{}/activities/{}", chore_list.id, activity.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { (t().chore()) }
                select #chore_id name="chore_id" required {
                    option disabled hidden value="" { }
                    @for chore in chores {
                        @if !chore.is_deleted() {
                            option value=(chore.id) selected[chore.id == activity.chore_id] {
                                (chore.name)
                                " (" (chore.points) "P)"
                            }
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

                button type="submit" { (t().update_action()) }
            }
        },
    )
}
