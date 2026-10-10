use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Auto,
    En,
    Fr,
}

impl Language {
    pub fn resolved(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Auto => resolve_system_locale(sys_locale::get_locale().as_deref()),
        }
    }
}

pub fn resolve_system_locale(locale: Option<&str>) -> &'static str {
    match locale.and_then(|value| value.split(['-', '_', '.']).next()) {
        Some(language) if language.eq_ignore_ascii_case("fr") => "fr",
        _ => "en",
    }
}
