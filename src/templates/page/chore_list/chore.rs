use maud::{html, Markup};
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::user;
use crate::domain::value::Date;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::ChoreListNavigationItem;

pub fn list(
    chore_list: chore_list::ChoreList,
    chores: Vec<chore::Chore>,
    deleted_chores: Vec<chore::Chore>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🧹")
            .title("Chores")
            .headline("🧹 Chores")
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url("/chore-lists")
            .meta_actions(html! {
                @if !chore_list.is_deleted() {
                    a.secondary.subtle href={ "/chore-lists/" (chore_list.id) "/chores/create" } { "+ Add" }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            ul.card-container.collapse {
                @for chore in chores {
                    li {
                        a.card href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) } {
                            div.title { (chore.name) }
                            small.text-muted {
                                (chore.points) "P"

                                @if let Some(next_due_date) = chore.next_due_date {
                                    @if next_due_date.is_today() || next_due_date.is_in_past() {
                                        " – "
                                        span.text-danger.fw-bold { "Due!" }
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
                    summary.arrow-left.text-muted { "Deleted Chores" }
                    ul.card-container.collapse {
                        @for chore in deleted_chores {
                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) } {
                                    div.title { (chore.name) }
                                    small.text-muted { (chore.points) "P" }
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
            .headline(&format!("🧹 {}", chore.name))
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url(&format!("/chore-lists/{}/chores", chore_list.id))
            .meta_actions(html! {
                @if chore.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="chore_restore" { "↻ Restore" }
                    form #chore_restore method="post" action={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) "/restore" } { }
                } @else if !chore_list.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="chore_delete" { "✗ Delete" }

                    a.secondary.subtle href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) "/update" } style="margin-left: 1.25rem;" { "✎ Edit" }

                    form #chore_delete method="post" action={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) "/delete" } { }
                }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            @if chore.is_deleted() || chore_list.is_deleted() {
                div {
                    em { "This chore has been deleted" }
                }

                br;
            }

            dl {
                dt { "Points" }
                dd { (chore.points) }

                @if let Some(interval_days) = chore.interval_days {
                    dt { "Interval" }
                    dd { "every " (interval_days) " day(s)" }
                }

                @if let Some(next_due_date) = chore.next_due_date {
                    @let is_due = next_due_date.is_today() || next_due_date.is_in_past();
                    dt { "Next Due Date" }
                    dd.text-danger[is_due].fw-bold[is_due] {
                        (next_due_date.format("%Y-%m-%d"))
                    }
                }

                @if let Some(description) = chore.description {
                    dt { "Description" }
                    dd { (description) }
                }
            }

            br;
            br;

            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    li {
                        a.card href={ "/chore-lists/" (chore_list.id) "/chores/" (chore.id) "/activities" } {
                            div.title { "✅ Activities" }
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
            .title("Create Chore")
            .headline("Create 🧹 Chore")
            .back_url(&format!("/chore-lists/{}/chores", chore_list.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            form method="post" {
                label for="name" { "Name" }
                input #name name="name" type="text" required;

                label for="description" {
                    "Description "
                    i.text-muted { "(optional)" }
                }
                textarea #description name="description" { }

                label for="points" { "Points" }
                input #points name="points" type="number" min="1" step="1" required;

                label for="interval_days" {
                    "Interval "
                    i.text-muted { "(optional)" }
                }
                div role="group" {
                    input #interval_days name="interval_days" type="number" min="1" step="1" aria-describedby="interval_days-help-text";
                    label for="interval_days" { "days" }
                }
                small #interval_days-help-text { "How often the chore should be done" }

                button type="submit" { "Create" }
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
            .title("Edit Chore")
            .headline("Edit 🧹 Chore")
            .back_url(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            form method="post" {
                label for="name" { "Name" }
                input #name name="name" type="text" required value=(chore.name);

                label for="description" {
                    "Description "
                    i.text-muted { "(optional)" }
                }
                textarea #description name="description" {
                    @if let Some(description) = chore.description {
                        (description)
                    }
                }

                label for="points" { "Points" }
                input #points name="points" type="number" min="1" step="1" required value=(chore.points);

                label for="interval_days" {
                    "Interval "
                    i.text-muted { "(optional)" }
                }
                div role="group" {
                    input #interval_days name="interval_days" type="number" min="1" step="1" aria-describedby="interval_days-help-text" value=[chore.interval_days];
                    label for="interval_days" { "days" }
                }
                small #interval_days-help-text { "How often the chore should be done" }

                button type="submit" { "Update" }
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
            .title("Activities")
            .headline("✅ Activities")
            .teaser(&format!("Of 🧹 {}", chore.name))
            .back_url(&format!("/chore-lists/{}/chores/{}", chore_list.id, chore.id))
            .meta_actions(html! {
                a.secondary.subtle href={ "/chore-lists/" (chore_list.id) "/activities/create" } { "+ Add" }
            })
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Chores)))
            .build(),
        html! {
            div.timeline {
                @for (date, activities_of_date) in activities_by_date {
                    div.timeline-date-separator { (date.format("%Y-%m-%d")) }
                    ul.card-container.collapse {
                        @for activity in activities_of_date {
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (user.name) }

                                    @if activity.comment.is_some() {
                                        small.text-muted { "Has comment" }
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
                            @let user = users.iter().find(|user| user.id == activity.user_id).unwrap();

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (user.name) }

                                    small.text-muted {
                                        (activity.date.format("%Y-%m-%d"))

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
