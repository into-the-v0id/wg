use maud::{html, Markup};
use crate::handler::settings::SettingsIndexPath;
use crate::handler::user::UserCreatePath;
use crate::handler::user::UserDeletePath;
use crate::handler::user::UserDetailPath;
use crate::handler::user::UserIndexPath;
use crate::handler::user::UserRestorePath;
use wg_core::model::user;
use crate::template::helper::t;
use crate::template::layout;
use crate::template::partial;
use crate::template::partial::navigation::GlobalNavigationItem;

pub fn list(
    users: Vec<user::User>,
    deleted_users: Vec<user::User>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title(&t().users())
            .headline(&format!("👤 {}", t().users()))
            .back_url(SettingsIndexPath.to_string().as_str())
            .meta_actions(html! {
                a.secondary.subtle href=(UserCreatePath) { "+ " (t().add_action()) }
            })
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            ul.card-container.collapse {
                @for user in users {
                    li {
                        a.card href=(UserDetailPath { user_id: user.id }) {
                            div.title { (user.name) }
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
                                a.card href=(UserDetailPath { user_id: user.id }) {
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

pub fn detail(user: user::User) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title(&user.name)
            .headline(&format!("👤 {}", user.name))
            .back_url(UserIndexPath.to_string().as_str())
            .meta_actions(html! {
                @if user.is_deleted() {
                    button.link.secondary.subtle.mb-0 type="submit" form="user_restore" { "↻ " (t().restore_action()) }
                    form #user_restore method="post" action=(UserRestorePath { user_id: user.id }) { }
                } @else {
                    button.link.secondary.subtle.mb-0 type="submit" form="user_delete" { "✗ " (t().delete_action()) }
                    form #user_delete method="post" action=(UserDeletePath { user_id: user.id }) { }
                }
            })
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            @if user.is_deleted() {
                div {
                    em { (t().user_has_been_deleted()) }
                }
            }
        },
    )
}

pub fn create() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title(&t().create_user())
            .headline(&format!("👤 {}", t().create_user()))
            .back_url(UserIndexPath.to_string().as_str())
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            form method="post" {
                label for="name" { (t().name()) }
                input #name name="name" type="text" required autocomplete="given-name";

                label for="handle" { (t().username()) }
                input #handle name="handle" type="text" required autocomplete="username";

                label for="password" { (t().password()) }
                input #password name="password" type="password" required minlength="5" autocomplete="current-password";

                button type="submit" { (t().create_action()) }
            }
        },
    )
}

pub fn update(user: user::User) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🪪")
            .title(&t().edit_profile())
            .headline(&t().edit_profile())
            .back_url(SettingsIndexPath.to_string().as_str())
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            form method="post" {
                label for="name" { (t().name()) }
                input #name name="name" type="text" required autocomplete="given-name" value=(user.name);

                label for="handle" { (t().username()) }
                input #handle name="handle" type="text" required autocomplete="username" value=(user.handle);

                label for="password" {
                    (t().new_password())
                    " "
                    i.text-muted { "(" (t().optional()) ")" }
                }
                input #password name="password" type="password" minlength="5" autocomplete="new-password" value="";

                button type="submit" { (t().update_action()) }
            }
        },
    )
}
