use maud::{html, Markup};
use crate::templates::helper::t;
use crate::domain::chore_list;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GlobalNavigationItem {
    ChoreLists,
    Settings,
}

pub fn global(active_item: Option<GlobalNavigationItem>) -> Markup {
    html! {
        ul {
            li {
                a href="/chore-lists" aria-current=[if active_item == Some(GlobalNavigationItem::ChoreLists) { Some("page") } else { None }] {
                    div.icon { "📋" }
                    div.label { (t().chore_lists()) }
                }
            }
            li {
                a href="/settings" aria-current=[if active_item == Some(GlobalNavigationItem::Settings) { Some("page") } else { None }] {
                    div.icon { "⚙️" }
                    div.label { (t().settings()) }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ChoreListNavigationItem {
    Activities,
    Chores,
    Users,
    Settings,
}

pub fn chore_list(chore_list: &chore_list::ChoreList, active_item: Option<ChoreListNavigationItem>) -> Markup {
    html! {
        ul {
            li {
                a href={ "/chore-lists/" (chore_list.id) "/activities" } aria-current=[if active_item == Some(ChoreListNavigationItem::Activities) { Some("page") } else { None }] {
                    div.icon { "✅" }
                    div.label { (t().activities()) }
                }
            }
            li {
                a href={ "/chore-lists/" (chore_list.id) "/chores" } aria-current=[if active_item == Some(ChoreListNavigationItem::Chores) { Some("page") } else { None }] {
                    div.icon { "🧹" }
                    div.label { (t().chores()) }
                }
            }
            li {
                a href={ "/chore-lists/" (chore_list.id) "/users" } aria-current=[if active_item == Some(ChoreListNavigationItem::Users) { Some("page") } else { None }] {
                    div.icon { "👤" }
                    div.label { (t().users()) }
                }
            }
            li {
                a href={ "/chore-lists/" (chore_list.id) "/settings" } aria-current=[if active_item == Some(ChoreListNavigationItem::Settings) { Some("page") } else { None }] {
                    div.icon { "⚙️" }
                    div.label { (t().settings()) }
                }
            }
        }
    }
}
