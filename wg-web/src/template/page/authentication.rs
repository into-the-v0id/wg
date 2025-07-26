use maud::{html, Markup};
use crate::template::helper::t;
use crate::template::layout;

pub fn login() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title(&t().login())
            .build(),
        html! {
            form method="post" {
                label for="email" { (t().email_address()) }
                input #email name="email" type="email" required autocomplete="email" autofocus;

                label for="password" { (t().password()) }
                input #password name="password" type="password" required autocomplete="current-password";

                button type="submit" { (t().login_action()) }
            }
        },
    )
}
