use maud::{html, Markup};
use crate::domain::authentication_session::AuthenticationSession;
use crate::templates::helper::t;
use crate::templates::layout;
use crate::templates::partial;
use crate::templates::partial::navigation::GlobalNavigationItem;

pub fn settings(auth_session: AuthenticationSession) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("⚙️")
            .title(&t().settings())
            .headline(&format!("⚙️ {}", t().settings()))
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            nav style="flex-direction: column;" {
                h4 { (t().profile()) }
                ul.card-container.collapse {
                    li {
                        a.card href={ "/users/" (auth_session.user_id) "/update" } {
                            div.title { "🪪 " (t().edit_profile()) }
                        }
                    }
                    li {
                        button.card.text-align-left.mb-0 type="submit" form="logout" {
                            div.title { "🚪 " (t().logout_action()) }
                        }
                        form #logout method="post" action="/logout" { }
                    }
                }

                h4 { (t().instance()) }
                ul.card-container.collapse {
                    li {
                        a.card href="/users" {
                            div.title { "👤 " (t().users()) }
                        }
                    }
                }

                h4 { (t().legal()) }
                ul.card-container.collapse {
                    li {
                        a.card href="/legal/privacy-policy" rel="privacy-policy" target="_blank" {
                            div.title { "📜 " (t().privacy_policy()) }
                        }
                    }
                }
            }
        },
    )
}
