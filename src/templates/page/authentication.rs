use maud::{html, Markup};
use crate::templates::helper::t;
use crate::templates::layout;

pub fn login() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title(&t().login())
            .headline(&t().login())
            .build(),
        html! {
            form method="post" {
                label for="handle" { (t().username()) }
                input #handle name="handle" type="text" required autocomplete="username" autofocus;

                label for="password" { (t().password()) }
                input #password name="password" type="password" required autocomplete="current-password";

                button type="submit" { (t().login_action()) }
            }
        },
    )
}
