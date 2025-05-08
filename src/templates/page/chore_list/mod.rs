pub mod activity;
pub mod chore;
pub mod user;

use maud::{html, Markup};
use crate::domain::chore_list;
use crate::domain::chore_list::ScoreResetInterval;
use crate::templates::helper::t;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::ChoreListNavigationItem;
use crate::templates::partial::navigation::GlobalNavigationItem;
use strum::IntoEnumIterator;

pub fn list(
    chore_lists: Vec<chore_list::ChoreList>,
    deleted_chore_lists: Vec<chore_list::ChoreList>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("📋")
            .title(&t().chore_lists())
            .headline(&format!("📋 {}", t().chore_lists()))
            .meta_actions(html! {
                a.secondary.subtle href="/chore-lists/create" { "+ " (t().add_action()) }
            })
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::ChoreLists)))
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
                    summary.arrow-left.text-muted { (t().deleted_chore_lists()) }
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
            .title(&t().create_chore_list())
            .headline(&format!("📋 {}", t().create_chore_list()))
            .back_url("/chore-lists")
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::ChoreLists)))
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

                label for="score_reset_interval" { (t().score_reset_interval()) }
                select #score_reset_interval name="score_reset_interval" aria-describedby="score_reset_interval-help-text" required {
                    option selected disabled hidden value="" { }
                    @for score_reset_interval in chore_list::ScoreResetInterval::iter() {
                        option value=(score_reset_interval) {
                            @match score_reset_interval {
                                ScoreResetInterval::Monthly => (t().interval_monthly()),
                                ScoreResetInterval::Quaterly => (t().interval_quaterly()),
                                ScoreResetInterval::HalfYearly => (t().interval_half_yearly()),
                                ScoreResetInterval::Yearly => (t().interval_yearly()),
                                ScoreResetInterval::Never => (t().interval_never()),
                            }
                        }
                    }
                }
                small #score_reset_interval-help-text { (t().score_reset_interval_help_text()) }

                button type="submit" { (t().create_action()) }
            }
        },
    )
}

pub fn update(chore_list: chore_list::ChoreList) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("📋")
            .title(&t().edit_chore_list())
            .headline(&format!("📋 {}", t().edit_chore_list()))
            .back_url(&format!("/chore-lists/{}/settings", chore_list.id))
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Settings)))
            .build(),
        html! {
            form method="post" {
                label for="name" { (t().name()) }
                input #name name="name" type="text" required value=(chore_list.name);

                label for="description" {
                    (t().description())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                textarea #description name="description" {
                    @if let Some(description) = chore_list.description {
                        (description)
                    }
                }

                label for="score_reset_interval" { (t().score_reset_interval()) }
                select #score_reset_interval name="score_reset_interval" aria-describedby="score_reset_interval-help-text" required {
                    option disabled hidden value="" { }
                    @for score_reset_interval in chore_list::ScoreResetInterval::iter() {
                        option value=(score_reset_interval) selected[score_reset_interval == chore_list.score_reset_interval] {
                            @match score_reset_interval {
                                ScoreResetInterval::Monthly => (t().interval_monthly()),
                                ScoreResetInterval::Quaterly => (t().interval_quaterly()),
                                ScoreResetInterval::HalfYearly => (t().interval_half_yearly()),
                                ScoreResetInterval::Yearly => (t().interval_yearly()),
                                ScoreResetInterval::Never => (t().interval_never()),
                            }
                        }
                    }
                }
                small #score_reset_interval-help-text { (t().score_reset_interval_help_text()) }

                button type="submit" { (t().update_action()) }
            }
        },
    )
}

pub fn settings(chore_list: chore_list::ChoreList) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("⚙️")
            .title(&t().settings())
            .headline(&format!("⚙️ {}", t().settings()))
            .teaser(&t().of_x(format!("📋 {}", chore_list.name)))
            .back_url("/chore-lists")
            .navigation(partial::navigation::chore_list(&chore_list, Some(ChoreListNavigationItem::Settings)))
            .build(),
        html! {
            nav style="flex-direction: column;" {
                ul.card-container.collapse {
                    @if chore_list.is_deleted() {
                        li {
                            button.card.text-align-left.mb-0 type="submit" form="chore_list_restore" {
                                div.title { "♻️ " (t().restore_chore_list()) }
                            }
                            form #chore_list_restore method="post" action={ "/chore-lists/" (chore_list.id) "/restore" } { }
                        }
                    } @else {
                        li {
                            a.card href={ "/chore-lists/" (chore_list.id) "/update" } {
                                div.title { "✏️ " (t().edit_chore_list()) }
                            }
                        }
                        li {
                            button.card.text-align-left.mb-0 type="submit" form="chore_list_delete" {
                                div.title.text-danger { "🗑️ " (t().delete_chore_list()) }
                            }
                            form #chore_list_delete method="post" action={ "/chore-lists/" (chore_list.id) "/delete" } { }
                        }
                    }
                }
            }
        },
    )
}
