use maud::{html, Markup};
use crate::domain::chore_list;
use crate::domain::chore_activity;
use crate::domain::chore;
use crate::domain::user;
use crate::domain::value::Date;
use crate::domain::value::Uuid;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::ChoreListNavigationItem;

pub fn list(
    chore_list: chore_list::ChoreList,
    users: Vec<user::User>,
    deleted_users: Vec<user::User>,
    scores_by_user: Vec<(Uuid, i32)>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title("Users")
            .headline("👤 Users")
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url("/chore-lists")
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
            .build(),
        html! {
            ol.card-container.collapse {
                @for (user_id, score) in scores_by_user {
                    @let user = users.iter().find(|user| user.id == user_id).unwrap();

                    li {
                        a.card href={ "/chore-lists/" (chore_list.id) "/users/" (user_id) } {
                            div.title { (user.name) }
                            small.text-muted { "Score: " (score) }
                        }
                    }
                }
            }

            @if ! deleted_users.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { "Past Users" }
                    ul.card-container.collapse {
                        @for user in deleted_users {
                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/users/" (user.id) } {
                                    div.title { (user.name) }
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
    user: user::User,
    chore_list: chore_list::ChoreList,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title(&user.name)
            .headline(&format!("👤 {}", user.name))
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url(&format!("/chore-lists/{}/users", chore_list.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
            .build(),
        html! {
            @if user.is_deleted() || chore_list.is_deleted() {
                div {
                    em { "This user has been deleted" }
                }

                br;
            }

            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    li {
                        a.card href={ "/chore-lists/" (chore_list.id) "/users/" (user.id) "/activities" } {
                            div.title { "✅ Activities" }
                        }
                    }
                }
            }
        },
    )
}

pub fn list_activities(
    user: user::User,
    chore_list: chore_list::ChoreList,
    activities_by_date: Vec<(Date, Vec<&chore_activity::ChoreActivity>)>,
    deleted_activities: Vec<chore_activity::ChoreActivity>,
    chores: Vec<chore::Chore>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✅")
            .title("Activities")
            .headline("✅ Activities")
            .teaser(&format!("Of 👤 {} in 📋 {}", user.name, chore_list.name))
            .back_url(&format!("/chore-lists/{}/users/{}", chore_list.id, user.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
            .build(),
        html! {
            div.timeline {
                @for (date, activities_of_date) in activities_by_date {
                    div.timeline-date-separator { (date.format("%Y-%m-%d")) }
                    ul.card-container.collapse {
                        @for activity in activities_of_date {
                            @let chore = chores.iter().find(|chore| chore.id == activity.chore_id).unwrap();

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (chore.points) "P"

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

                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities/" (activity.id) } {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (chore.points) "P"

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
