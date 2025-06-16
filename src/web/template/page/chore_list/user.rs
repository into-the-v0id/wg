use maud::{html, Markup};
use crate::web::application::chore_activity::ChoreActivityDetailPath;
use crate::web::application::chore_list::ChoreListIndexPath;
use crate::web::application::chore_list_user::ChoreListUserActivitiesPath;
use crate::web::application::chore_list_user::ChoreListUserDetailPath;
use crate::web::application::chore_list_user::ChoreListUserIndexPath;
use crate::model::chore_list;
use crate::model::chore_activity;
use crate::model::chore;
use crate::model::user;
use crate::model::user::UserId;
use crate::value::Date;
use crate::web::template::helper::format_date_long;
use crate::web::template::helper::format_date_long_simple;
use crate::web::template::helper::t;
use crate::web::template::layout;
use crate::web::template::partial;
use crate::web::template::partial::navigation::ChoreListNavigationItem;

pub fn list(
    chore_list: chore_list::ChoreList,
    users: Vec<user::User>,
    deleted_users: Vec<user::User>,
    scores_by_user: Vec<(UserId, i32)>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title(&t().users())
            .headline(&format!("👤 {}", t().users()))
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreListIndexPath.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
            .build(),
        html! {
            ol.card-container.collapse {
                @for (user_id, score) in scores_by_user {
                    @let user = users.iter().find(|user| user.id == user_id).unwrap();

                    li {
                        a.card href=(ChoreListUserDetailPath { chore_list_id: chore_list.id, user_id: user.id }) {
                            div.title { (user.name) }
                            small.text-muted { (t().score_value(score)) }
                        }
                    }
                }
            }

            @if ! deleted_users.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { (t().past_users()) }
                    ul.card-container.collapse {
                        @for user in deleted_users {
                            li {
                                a.card href=(ChoreListUserDetailPath { chore_list_id: chore_list.id, user_id: user.id }) {
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
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url(ChoreListUserIndexPath { chore_list_id: chore_list.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
            .build(),
        html! {
            @if user.is_deleted() || chore_list.is_deleted() {
                div {
                    em { (t().user_has_been_deleted()) }
                }

                br;
            }

            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    li {
                        a.card href=(ChoreListUserActivitiesPath { chore_list_id: chore_list.id, user_id: user.id }) {
                            div.title { "✅ " (t().activities()) }
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
            .title(&t().activities())
            .headline(&format!("✅ {}", t().activities()))
            .teaser(&t().of_x_in_y(
                format!("👤 {}", user.name),
                format!("📋 {}", chore_list.name)
            ))
            .back_url(ChoreListUserDetailPath { chore_list_id: chore_list.id, user_id: user.id }.to_string().as_str())
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Users)))
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

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (t().points_value_short(chore.points))

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

                            li {
                                a.card href=(ChoreActivityDetailPath {chore_list_id: chore_list.id, chore_activity_id: activity.id }) {
                                    div.title { (chore.name) }

                                    small.text-muted {
                                        (t().points_value_short(chore.points))

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
