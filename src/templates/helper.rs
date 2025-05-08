use crate::{Translations, TRANSLATIONS};

pub fn t() -> Translations {
    TRANSLATIONS.get()
}
