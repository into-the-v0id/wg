use maud::{html, Markup};
use crate::domain::authentication_session::AuthenticationSession;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::GlobalNavigationItem;

pub fn settings(auth_session: AuthenticationSession) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("⚙️")
            .title("Settings")
            .headline("⚙️ Settings")
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            nav style="flex-direction: column;" {
                h4 { "Profile" }
                ul.card-container.collapse {
                    li {
                        a.card href={ "/users/" (auth_session.user_id) "/update" } {
                            div.title { "🪪 Edit Profile" }
                        }
                    }
                    li {
                        button.card.text-align-left.mb-0 type="submit" form="logout" {
                            div.title { "🚪 Logout" }
                        }
                        form #logout method="post" action="/logout" { }
                    }
                }

                h4 { "Instance" }
                ul.card-container.collapse {
                    li {
                        a.card href="/users" {
                            div.title { "👤 Users" }
                        }
                    }
                }

                h4 { "Legal" }
                ul.card-container.collapse {
                    li {
                        a.card href="/legal/privacy-policy" rel="privacy-policy" target="_blank" {
                            div.title { "📜 Privacy Policy" }
                        }
                    }
                }
            }
        },
    )
}
