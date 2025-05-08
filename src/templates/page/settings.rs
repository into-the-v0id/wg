use maud::{html, Markup};
use strum::IntoEnumIterator;
use crate::application::language::Language;
use crate::application::language::LanguageSelection;
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
                ul.card-container.collapse {
                    li {
                        a.card href={ "/settings/appearance" } {
                            div.title { "✨ " (t().appearance()) }
                        }
                    }
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

pub fn appearence(language_selection: LanguageSelection) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✨")
            .title(&t().appearance())
            .headline(&t().appearance())
            .back_url("/settings")
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            form method="post" {
                label for="language" { (t().language()) }
                select.auto-submit #language name="language" required {
                    option value=(LanguageSelection::Auto.to_string()) selected[language_selection == LanguageSelection::Auto] {
                        (t().langauge_auto())
                    }
                    @for language in Language::iter() {
                        option value=(language) selected[language_selection == LanguageSelection::Language(language)] {
                            @match language {
                                Language::EN => "English",
                                Language::DE => "Deutsch",
                            }
                        }
                    }
                }
            }
        },
    )
}
