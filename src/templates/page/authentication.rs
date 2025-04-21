use maud::{html, Markup};
use crate::templates::layout;

pub fn login() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title("Login")
            .headline("Login")
            .build(),
        html! {
            form method="post" {
                label for="handle" { "Handle" }
                input #handle name="handle" type="text" required autocomplete="username" autofocus;

                label for="password" { "Password" }
                input #password name="password" type="password" required autocomplete="current-password";

                button type="submit" { "Login" }
            }
        },
    )
}
