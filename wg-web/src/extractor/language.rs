use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::extract::CookieJar;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Language(pub wg_core::value::Language);

pub const DEFAULT_LANGAGE: wg_core::value::Language = wg_core::value::Language::EN;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LanguageSelection {
    Auto,
    Language(wg_core::value::Language),
}

impl FromStr for LanguageSelection {
    type Err = <wg_core::value::Language as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "auto" => Ok(LanguageSelection::Auto),
            _ => wg_core::value::Language::from_str(string).map(LanguageSelection::Language),
        }
    }
}

impl ToString for LanguageSelection {
    fn to_string(&self) -> String {
        match self {
            LanguageSelection::Auto => "auto".to_string(),
            LanguageSelection::Language(language) => language.to_string(),
        }
    }
}

pub const COOKIE_NAME: &str = "language";

impl <S> FromRequestParts<S> for LanguageSelection
where
    S: Send + Sync
{
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = parts.extract::<CookieJar>().await.unwrap();

        let raw_langauage = cookie_jar.get(COOKIE_NAME)
            .map(|value| value.value());
        let raw_langauage = match raw_langauage {
            Some(raw_langauage) => raw_langauage,
            None => return Ok(LanguageSelection::Auto),
        };

        let selection = LanguageSelection::from_str(raw_langauage)
            .unwrap_or(LanguageSelection::Auto);

        Ok(selection)
    }
}

impl <S> FromRequestParts<S> for Language
where
    S: Send + Sync
{
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let selection = parts.extract::<LanguageSelection>().await.unwrap();

        if let LanguageSelection::Language(language) = selection {
            return Ok(Language(language));
        }

        let accept_language_header = parts.headers.get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok());
        let accept_language_header = match accept_language_header {
            Some(accept_language_header) => accept_language_header,
            None => return Ok(Language(DEFAULT_LANGAGE)),
        };

        let requested_languages = accept_language::parse_with_quality(accept_language_header);
        let matching_languages = requested_languages.into_iter()
            .filter_map(|(req_langauge, quality)| match wg_core::value::Language::from_str(&req_langauge) {
                Ok(available_language) => Some((available_language, quality)),
                Err(_) => None,
            })
            .collect::<Vec<(wg_core::value::Language, f32)>>();
        let best_language = matching_languages.into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(language, _)| language);
        let best_language = match best_language {
            Some(best_language) => best_language,
            None => return Ok(Language(DEFAULT_LANGAGE)),
        };

        Ok(Language(best_language))
    }
}
