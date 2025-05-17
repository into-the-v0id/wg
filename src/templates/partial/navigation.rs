use maud::{html, Markup};
use crate::application::chore::ChoreIndexPath;
use crate::application::chore_activity::ChoreActivityIndexPath;
use crate::application::chore_list::{ChoreListIndexPath, ChoreListSettingsPath};
use crate::application::chore_list_user::ChoreListUserIndexPath;
use crate::application::settings::SettingsIndexPath;
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
                a href=(ChoreListIndexPath) aria-current=[if active_item == Some(GlobalNavigationItem::ChoreLists) { Some("page") } else { None }] {
                    div.icon { "📋" }
                    div.label { (t().chore_lists()) }
                }
            }
            li {
                a href=(SettingsIndexPath) aria-current=[if active_item == Some(GlobalNavigationItem::Settings) { Some("page") } else { None }] {
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
                a href=(ChoreActivityIndexPath {chore_list_id: chore_list.id }) aria-current=[if active_item == Some(ChoreListNavigationItem::Activities) { Some("page") } else { None }] {
                    div.icon { "✅" }
                    div.label { (t().activities()) }
                }
            }
            li {
                a href=(ChoreIndexPath { chore_list_id: chore_list.id }) aria-current=[if active_item == Some(ChoreListNavigationItem::Chores) { Some("page") } else { None }] {
                    div.icon { "🧹" }
                    div.label { (t().chores()) }
                }
            }
            li {
                a href=(ChoreListUserIndexPath { chore_list_id: chore_list.id }) aria-current=[if active_item == Some(ChoreListNavigationItem::Users) { Some("page") } else { None }] {
                    div.icon { "👤" }
                    div.label { (t().users()) }
                }
            }
            li {
                a href=(ChoreListSettingsPath { chore_list_id: chore_list.id }) aria-current=[if active_item == Some(ChoreListNavigationItem::Settings) { Some("page") } else { None }] {
                    div.icon { "⚙️" }
                    div.label { (t().settings()) }
                }
            }
        }
    }
}
