use bon::Builder;
use maud::{html, Markup, PreEscaped};
use crate::web::application::theme::Theme;
use crate::web::template::helper::t;
use crate::{web::application::assets, LANGUAGE, THEME};

fn emoji_favicon(emoji: &str) -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" {
            text x="50" y="58" dominant-baseline="middle" text-anchor="middle" fill="black" style="font-size:89px;line-height:1;text-align:center;" {
                (emoji)
            }
        }
    }
}

#[derive(Builder)]
pub struct BlankLayoutOptions<'a> {
    emoji: Option<&'a str>,
    title: &'a str,
    head: Option<Markup>,
}

pub fn blank(
    options: BlankLayoutOptions,
    content: Markup,
) -> Markup {
    let theme = THEME.get();

    html! {
        (maud::DOCTYPE)
        html lang=(LANGUAGE.get().to_string()) data-theme=[if theme == Theme::Auto { None } else { Some(theme) }] {
            head {
                meta charset="utf-8";
                title { (options.title) }
                meta name="viewport" content="width=device-width, initial-scale=1";
                @match theme {
                    Theme::Auto => meta name="color-scheme" content="dark light";
                    Theme::Light => meta name="color-scheme" content="light";
                    Theme::Dark => meta name="color-scheme" content="dark";
                }

                link rel="preload" href=(assets::get_url("/css/pico.css").unwrap()) as="style";
                link rel="preload" href=(assets::get_url("/css/app.css").unwrap()) as="style";
                link rel="preload" href=(assets::get_url("/js/app.js").unwrap()) as="script";

                link rel="stylesheet" href=(assets::get_url("/css/pico.css").unwrap());
                link rel="stylesheet" href=(assets::get_url("/css/app.css").unwrap());

                link rel="icon" type="image/svg+xml" href={ "data:image/svg+xml;utf8," (emoji_favicon(options.emoji.unwrap_or("🏠")).into_string()) };
                meta name="format-detection" content="telephone=no";
                meta name="msapplication-tap-highlight" content="no";

                meta name="robots" content="noindex, nofollow";

                link rel="manifest" href=(assets::get_url("/manifest.json").unwrap());
                meta name="application-name" content="WG";
                meta name="apple-mobile-web-app-title" content="WG";
                meta name="theme-color" content="#13171F";
                meta name="mobile-web-app-capable" content="yes";

                @if let Some(head) = options.head {
                    (head)
                }
            }
            body {
                (content)

                script src=(assets::get_url("/js/app.js").unwrap()) defer {}
            }
        }
    }
}

#[derive(Builder)]
pub struct DefaultLayoutOptions<'a> {
    emoji: Option<&'a str>,
    title: &'a str,
    headline: &'a str,
    teaser: Option<&'a str>,
    back_url: Option<&'a str>,
    meta_actions: Option<Markup>,
    navigation: Option<Markup>,
    head: Option<Markup>,
}

pub fn default(
    options: DefaultLayoutOptions,
    content: Markup,
) -> maud::Markup {
    blank(
        BlankLayoutOptions::builder()
            .maybe_emoji(options.emoji)
            .title(options.title)
            .maybe_head(options.head)
            .build(),
        html! {
            div.container.layout.py-block {
                div.layout-left { }
                header.layout-center {
                    div style="display: flex; justify-content: space-between;" {
                        div {
                            @if let Some(back_url) = options.back_url {
                                a.secondary.subtle href=(back_url) rel="parent" {
                                    (PreEscaped("&larr;")) " " (t().back_action())
                                }
                            } @else {
                                (PreEscaped("&nbsp;"))
                            }
                        }
                        div {
                            @if let Some(meta_actions) = options.meta_actions {
                                (meta_actions)
                            } @else {
                                (PreEscaped("&nbsp;"))
                            }
                        }
                    }

                    br;

                    @if let Some(teaser) = options.teaser {
                        hgroup {
                            h1 { (options.headline) }
                            p style="margin-top: 0.25rem;" { (teaser) }
                        }
                    } @else {
                        h1 { (options.headline) }
                    }
                }
                div.layout-right { }
            }

            div.container.layout.py-block {
                aside.layout-left {
                    @if let Some(navigation) = &options.navigation {
                        nav.side-nav.desktop-only {
                            (navigation)
                        }
                    }
                }
                main.layout-center {
                    (content)
                }
                div.layout-right { }
            }

            @if let Some(navigation) = &options.navigation {
                nav.nav-bar.mobile-only {
                    (navigation)
                }
                div.nav-bar-spacer.mobile-only { }
            }
        }
    )
}
