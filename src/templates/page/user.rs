use maud::{html, Markup};
use crate::domain::user;
use crate::templates::layout;

pub fn list(
    users: Vec<user::User>,
    deleted_users: Vec<user::User>,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title("Users")
            .headline("👤 Users")
            .back_url("/settings")
            .meta_actions(html! {
                a.secondary.text-decoration-none.underline-on-hover href="/users/create" { "+ Add" }
            })
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" aria-current="page" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
            .build(),
        html! {
            ul.card-container.collapse {
                @for user in users {
                    li {
                        a.card href={ "/users/" (user.id) } {
                            div.title { (user.name) }
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
                                a.card href={ "/users/" (user.id) } {
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
            .back_url("/users")
            .meta_actions(html! {
                @if user.is_deleted() {
                    button.link.secondary.text-decoration-none.underline-on-hover.mb-0 type="submit" form="user_restore" { "↻ Restore" }
                    form #user_restore method="post" action={ "/users/" (user.id) "/restore" } { }
                } @else {
                    button.link.secondary.text-decoration-none.underline-on-hover.mb-0 type="submit" form="user_delete" { "✗ Delete" }
                    form #user_delete method="post" action={ "/users/" (user.id) "/delete" } { }
                }
            })
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" aria-current="page" {
                            div.icon { "⚙️" }
                            div.label { "Settings" }
                        }
                    }
                }
            })
            .build(),
        html! {
            @if user.is_deleted() {
                div {
                    em { "This user has been deleted" }
                }
            }
        },
    )
}

pub fn create() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("👤")
            .title("Create User")
            .headline("Create 👤 User")
            .back_url("/users")
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" aria-current="page" {
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
                input #name name="name" type="text" required autocomplete="given-name";

                label for="handle" { "Handle" }
                input #handle name="handle" type="text" required autocomplete="username";

                label for="password" { "Password" }
                input #password name="password" type="password" required minlength="5" autocomplete="current-password";

                button type="submit" { "Create" }
            }
        },
    )
}

pub fn update(user: user::User) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("🪪")
            .title("Edit Profile")
            .headline("Edit Profile")
            .back_url("/settings")
            .navigation(html! {
                ul {
                    li {
                        a href="/chore-lists" {
                            div.icon { "📋" }
                            div.label { "Chore Lists" }
                        }
                    }
                    li {
                        a href="/settings" aria-current="page" {
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
                input #name name="name" type="text" required autocomplete="given-name" value=(user.name);

                label for="handle" { "Handle" }
                input #handle name="handle" type="text" required autocomplete="username" value=(user.handle);

                label for="password" {
                    "New Password "
                    i.text-muted { "(optional)" }
                }
                input #password name="password" type="password" minlength="5" autocomplete="new-password" value="";

                button type="submit" { "Update" }
            }
        },
    )
}
