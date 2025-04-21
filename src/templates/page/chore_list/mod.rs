pub mod activity;
pub mod chore;
pub mod user;

use maud::{html, Markup};
use crate::domain::chore_list;
use crate::templates::layout;
use strum::IntoEnumIterator;

pub fn list(
    chore_lists: Vec<chore_list::ChoreList>,
    deleted_chore_lists: Vec<chore_list::ChoreList>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("📋")
            .title("Chore Lists")
            .headline("📋 Chore Lists")
            .meta_actions(html! {
                a.secondary.text-decoration-none.underline-on-hover href="/chore-lists/create" { "+ Add" }
            })
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" aria-current="page" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
            .build(),
        html! {
            ul.card-container.collapse {
                @for chore_list in chore_lists {
                    li {
                        a.card href={ "/chore-lists/" (chore_list.id) "/activities" } {
                            div.title { (chore_list.name) }

                            @if let Some(description) = chore_list.description {
                                small.text-muted.text-elipsis { (description) }
                            }
                        }
                    }
                }
            }

            @if ! deleted_chore_lists.is_empty() {
                br;

                details {
                    summary.arrow-left.text-muted { "Deleted Chore Lists" }
                    ul.card-container.collapse {
                        @for chore_list in deleted_chore_lists {
                            li {
                                a.card href={ "/chore-lists/" (chore_list.id) "/activities" } {
                                    div.title { (chore_list.name) }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn create() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("📋")
            .title("Create Chore List")
            .headline("Create 📋 Chore List")
            .back_url("/chore-lists")
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" aria-current="page" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
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

                label for="score_reset_interval" { "Score Reset Interval" }
                select #score_reset_interval name="score_reset_interval" aria-describedby="score_reset_interval-help-text" required {
                    option selected disabled hidden value="" { }
                    @for score_reset_interval in chore_list::ScoreResetInterval::iter() {
                        option value=(score_reset_interval) { (score_reset_interval) }
                    }
                }
                small #score_reset_interval-help-text { "Reset the score of all users in the specified interval" }

                button type="submit" { "Create" }
            }
        },
    )
}

pub fn update(chore_list: chore_list::ChoreList) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("📋")
            .title("Edit Chore List")
            .headline("Edit 📋 Chore List")
            .back_url(&format!("/chore-lists/{}/settings", chore_list.id))
            .navigation(html! {
                ul {
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/activities" } {
                            div.icon { "✅" }
                            div.label { "Activities" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/chores" } {
                            div.icon { "🧹" }
                            div.label { "Chores" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/users" } {
                            div.icon { "👤" }
                            div.label { "Users" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/settings" } aria-current="page" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
            .build(),
        html! {
            form method="post" {
                label for="name" { "Name" }
                input #name name="name" type="text" required value=(chore_list.name);

                label for="description" {
                    "Description "
                    i.text-muted { "(optional)" }
                }
                textarea #description name="description" {
                    @if let Some(description) = chore_list.description {
                        (description)
                    }
                }

                label for="score_reset_interval" { "Score Reset Interval" }
                select #score_reset_interval name="score_reset_interval" aria-describedby="score_reset_interval-help-text" required {
                    option disabled hidden value="" { }
                    @for score_reset_interval in chore_list::ScoreResetInterval::iter() {
                        option value=(score_reset_interval) selected[score_reset_interval == chore_list.score_reset_interval] { (score_reset_interval) }
                    }
                }
                small #score_reset_interval-help-text { "Reset the score of all users in the specified interval" }

                button type="submit" { "Update" }
            }
        },
    )
}

pub fn settings(chore_list: chore_list::ChoreList) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("⚙️")
            .title("Settings")
            .headline("⚙️ Settings")
            .teaser(&format!("Of 📋 {}", chore_list.name))
            .back_url("/chore-lists")
            .navigation(html! {
                ul {
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/activities" } {
                            div.icon { "✅" }
                            div.label { "Activities" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/chores" } {
                            div.icon { "🧹" }
                            div.label { "Chores" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/users" } {
                            div.icon { "👤" }
                            div.label { "Users" }
                        }
                    }
                    li {
                        a href={ "/chore-lists/" (chore_list.id) "/settings" } aria-current="page" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
            .build(),
        html! {
            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    @if chore_list.is_deleted() {
                        li {
                            button.card.text-align-left.mb-0 type="submit" form="chore_list_restore" {
                                div.title { "♻️ Restore Chore List" }
                            }
                            form #chore_list_restore method="post" action={ "/chore-lists/" (chore_list.id) "/restore" } { }
                        }
                    } @else {
                        li {
                            a.card href={ "/chore-lists/" (chore_list.id) "/update" } {
                                div.title { "✏️ Edit Chore List" }
                            }
                        }
                        li {
                            button.card.text-align-left.mb-0 type="submit" form="chore_list_delete" {
                                div.title.text-danger { "🗑️ Delete Chore List" }
                            }
                            form #chore_list_delete method="post" action={ "/chore-lists/" (chore_list.id) "/delete" } { }
                        }
                    }
                }
            }
        },
    )
}
