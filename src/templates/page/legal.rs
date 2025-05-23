use maud::{html, Markup};
use crate::templates::helper::t;
use crate::templates::layout;

pub fn privacy_policy() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title(&t().privacy_policy())
            .headline(&t().privacy_policy())
            .back_url("/")
            .build(),
        html! {
            @let text = t().privacy_policy_text();
            @for line in text.lines() {
                p { (line) }
            }
        },
    )
}
