use maud::{html, Markup};
use crate::templates::layout;

pub fn privacy_policy() -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .title("Privacy Policy")
            .headline("Privacy Policy")
            .back_url("/")
            .build(),
        html! {
            p {
                "Data entered in this Web App can only be archived - it cannot be fully deleted by the user"
                br;
                "Any changes made by a user are logged and associated with that user account"
                br;
                "If you wish for your data to be removed, then please contact your admin."
            }

            p {
                "Access Logs (such as IP Addresses and User-Agents) may be collected for security and debugging purposes only."
            }
        },
    )
}
