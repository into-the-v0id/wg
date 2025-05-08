use axum::{response::{IntoResponse, Redirect}, Form};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use maud::Markup;
use serde_with::serde_as;
use crate::{domain::authentication_session::AuthenticationSession, templates};
use super::{language::{self, LanguageSelection}, theme::{self, Theme}};

pub async fn view(auth_session: AuthenticationSession) -> Markup {
    templates::page::settings::settings(auth_session)
}

pub async fn view_appearance_form(
    language_selection: LanguageSelection,
    theme_selection: Theme,
    _auth_session: AuthenticationSession,
) -> Markup {
    templates::page::settings::appearence(language_selection, theme_selection)
}

#[serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct AppearancePayload {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    language: LanguageSelection,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    theme: Theme,
}

pub async fn update_appearance(
    mut cookie_jar: CookieJar,
    _auth_session: AuthenticationSession,
    Form(payload): Form<AppearancePayload>,
) -> impl IntoResponse {
    let language_cookie = Cookie::build((language::COOKIE_NAME, payload.language.to_string()))
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .permanent()
        .build();
    cookie_jar = cookie_jar.remove(language::COOKIE_NAME);
    cookie_jar = cookie_jar.add(language_cookie);

    let theme_cookie = Cookie::build((theme::COOKIE_NAME, payload.theme.to_string()))
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .path("/")
        .permanent()
        .build();
    cookie_jar = cookie_jar.remove(theme::COOKIE_NAME);
    cookie_jar = cookie_jar.add(theme_cookie);

    (cookie_jar, Redirect::to("/settings/appearance"))
}
