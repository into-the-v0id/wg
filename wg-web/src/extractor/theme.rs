use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::extract::CookieJar;
use std::str::FromStr;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
    strum::IntoStaticStr,
    strum::EnumIter,
)]
pub enum Theme {
    #[strum(serialize = "auto")]
    Auto,
    #[strum(serialize = "light")]
    Light,
    #[strum(serialize = "dark")]
    Dark,
}

pub const COOKIE_NAME: &str = "theme";

impl <S> FromRequestParts<S> for Theme
where
    S: Send + Sync
{
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();

        let raw_theme = cookie_jar.get(COOKIE_NAME)
            .map(|value| value.value());
        let raw_theme = match raw_theme {
            Some(raw_theme) => raw_theme,
            None => return Ok(Theme::Auto),
        };

        let theme = Theme::from_str(raw_theme)
            .unwrap_or(Theme::Auto);

        Ok(theme)
    }
}
