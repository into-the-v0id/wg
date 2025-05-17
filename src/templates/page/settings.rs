use maud::{html, Markup};
use strum::IntoEnumIterator;
use crate::application::authentication::LogoutPath;
use crate::application::language::Language;
use crate::application::language::LanguageSelection;
use crate::application::legal::PrivacyPolicyPath;
use crate::application::settings::SettingsAppearancePath;
use crate::application::settings::SettingsIndexPath;
use crate::application::theme::Theme;
use crate::application::user::UserIndexPath;
use crate::application::user::UserUpdatePath;
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
                        a.card href=(SettingsAppearancePath) {
                            div.title { "✨ " (t().appearance()) }
                        }
                    }
                    li {
                        a.card href=(UserUpdatePath { user_id: auth_session.user_id }) {
                            div.title { "🪪 " (t().edit_profile()) }
                        }
                    }
                    li {
                        button.card.text-align-left.mb-0 type="submit" form="logout" {
                            div.title { "🚪 " (t().logout_action()) }
                        }
                        form #logout method="post" action=(LogoutPath) { }
                    }
                }

                h4 { (t().instance()) }
                ul.card-container.collapse {
                    li {
                        a.card href=(UserIndexPath) {
                            div.title { "👤 " (t().users()) }
                        }
                    }
                }

                h4 { (t().legal()) }
                ul.card-container.collapse {
                    li {
                        a.card href=(PrivacyPolicyPath) rel="privacy-policy" target="_blank" {
                            div.title { "📜 " (t().privacy_policy()) }
                        }
                    }
                }
            }
        },
    )
}

pub fn appearence(
    language_selection: LanguageSelection,
    theme_seleciton: Theme,
) -> Markup {
    layout::default(
        layout::DefaultLayoutOptions::builder()
            .emoji("✨")
            .title(&t().appearance())
            .headline(&t().appearance())
            .back_url(SettingsIndexPath.to_string().as_str())
            .navigation(partial::navigation::global(Some(GlobalNavigationItem::Settings)))
            .build(),
        html! {
            form method="post" {
                label for="language-selection" { (t().language()) }
                select.auto-submit #language-selection name="language" required {
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

                label for="theme-selection" { (t().theme()) }
                select.auto-submit #theme-selection name="theme" required {
                    @for theme in Theme::iter() {
                        option value=(theme) selected[theme_seleciton == theme] {
                            @match theme {
                                Theme::Auto => (t().theme_auto()),
                                Theme::Light => (t().theme_light()),
                                Theme::Dark => (t().theme_dark()),
                            }
                        }
                    }
                }
            }
        },
    )
}
