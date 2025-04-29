use maud::{html, Markup};
use crate::domain::authentication_session::AuthenticationSession;
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::user;
use crate::domain::value::Date;
use crate::domain::value::DateTime;
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
            .title("Activities")
            .headline("✅ Activities")
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url("/chore-lists")
            .meta_actions(html! {
                @if !chore_list.is_deleted() {
                    a.secondary.text-decoration-none.underline-on-hover href={ "/chore-lists/" (chore_list.id) "/activities/create" } { "+ Add" }
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
                                        (chore.points) "P"

                                        " – " (user.name)

                                        @if activity.comment.is_some() {
                                            " – Has comment"
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
                    summary.arrow-left.text-muted { "Deleted Activities" }
                    ul.card-container.collapse {
                        @for activity in deleted_activities {
                            @let chore = chores.iter().find(|chore| chore.id == activity.chore_id).unwrap();
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (chore.points) "P"

                                        " – " (user.name)

                                        " – " (activity.date.format("%Y-%m-%d"))

                                        @if activity.comment.is_some() {
                                            " – Has comment"
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
            .title("Activity")
            .headline("✅ Activity")
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url(&format!("/chore-lists/{}/activities", chore_list.id))
            .meta_actions(html! {
                @if activity.is_deleted() {
                    button.link.secondary.text-decoration-none.underline-on-hover.mb-0 type="submit" form="activity_restore" { "↻ Restore" }
                    form #activity_restore method="post" action={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) "/restore" } { }
                } @else if !chore.is_deleted() && !chore_list.is_deleted() && activity.user_id == auth_session.user_id {
                    button.link.secondary.text-decoration-none.underline-on-hover.mb-0 type="submit" form="activity_delete" { "✗ Delete" }

                    @if allow_edit {
                        a.secondary.text-decoration-none.underline-on-hover href="/chore-lists/{{ chore_list.id }}/activities/{{ activity.id }}/update" style="margin-left: 1.25rem;" { "✎ Edit" }
                    }

                    form #activity_delete method="post" action="/chore-lists/{{ chore_list.id }}/activities/{{ activity.id }}/delete" { }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            @if activity.is_deleted() || chore.is_deleted() || chore_list.is_deleted() {
                div {
                    em { "This activity has been deleted" }
                }

                br;
            }

            dl {
                dt { "Date" }
                dd { (activity.date.format("%Y-%m-%d")) }

                dt { "User" }
                dd { a.inherit.text-decoration-none.underline-on-hover href={ "/chore-lists/" (chore_list.id) "/users/" (user.id) } { "👤 " (user.name) } }

                dt { "Chore" }
                dd {
                    a.inherit.text-decoration-none.underline-on-hover href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) } {
                        "🧹 " (chore.name) " (" (chore.points) "P)"
                    }
                }

                @if let Some(comment) = activity.comment {
                    dt { "Comment" }
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
            .title("Create Activity")
            .headline("Create ✅ Activity")
            .back_url(&format!("/chore-lists/{}/activities", chore_list.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { "Chore" }
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

                label for="date" { "Date" }
                input #date name="date" type="date" min=(min_date.format("%Y-%m-%d")) max=(max_date.format("%Y-%m-%d")) value=(now.format("%Y-%m-%d")) required;

                label for="comment" {
                    "Comment "
                    i.text-muted { "(optional)" }
                }
                textarea #comment name="comment" { }

                button type="submit" { "Create" }
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
            .title("Edit Activity")
            .headline("Edit ✅ Activity")
            .back_url(&format!("/chore-lists/{}/activities/{}", chore_list.id, activity.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Activities)))
            .build(),
        html! {
            form method="post" {
                label for="chore_id" { "Chore" }
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

                label for="date" { "Date" }
                input #date name="date" type="date" min=(min_date.format("%Y-%m-%d")) max=(max_date.format("%Y-%m-%d")) value=(activity.date.format("%Y-%m-%d")) required;

                label for="comment" {
                    "Comment "
                    i.text-muted { "(optional)" }
                }
                textarea #comment name="comment" {
                    @if let Some(comment) = activity.comment {
                        (comment)
                    }
                }

                button type="submit" { "Update" }
            }
        },
    )
}
