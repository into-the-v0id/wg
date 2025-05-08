use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
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
pub enum Language {
    #[strum(serialize = "en")]
    EN,
    #[strum(serialize = "de")]
    DE
}

pub const DEFAULT_LANGAGE: Language = Language::EN;

impl <S> FromRequestParts<S> for Language
where
    S: Send + Sync
{
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let accept_language_header = parts.headers.get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok());
        let accept_language_header = match accept_language_header {
            Some(accept_language_header) => accept_language_header,
            None => return Ok(DEFAULT_LANGAGE),
        };

        let requested_languages = accept_language::parse_with_quality(accept_language_header);
        let matching_languages = requested_languages.into_iter()
            .filter_map(|(req_langauge, quality)| match Language::from_str(&req_langauge) {
                Ok(available_language) => Some((available_language, quality)),
                Err(_) => None,
            })
            .collect::<Vec<(Language, f32)>>();
        let best_language = matching_languages.into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(language, _)| language);
        let best_language = match best_language {
            Some(best_language) => best_language,
            None => return Ok(DEFAULT_LANGAGE),
        };

        Ok(best_language)
    }
}
