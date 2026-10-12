use serde::{Deserialize, Serialize};

macro_rules! languages {
    ($($variant:ident = $tag:literal),* $(,)?) => {
        /// ! Serde names are the BCP 47 tags shared with the interface's `Language`
        /// ! enum, the Paraglide locales and the rust-i18n files: the lists must
        /// ! stay identical.
        #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
        pub enum Language {
            #[default]
            #[serde(rename = "auto")]
            Auto,
            $(#[serde(rename = $tag)] $variant,)*
        }

        impl Language {
            pub const SUPPORTED: &'static [&'static str] = &[$($tag),*];

            fn tag(self) -> Option<&'static str> {
                match self {
                    Self::Auto => None,
                    $(Self::$variant => Some($tag),)*
                }
            }
        }
    };
}

languages! {
    En = "en",
    Fr = "fr",
    De = "de",
    EsEs = "es-ES",
    Es419 = "es-419",
    PtPt = "pt-PT",
    PtBr = "pt-BR",
    It = "it",
    Nl = "nl",
    Pl = "pl",
    Ru = "ru",
    Uk = "uk",
    Cs = "cs",
    Hu = "hu",
    Ro = "ro",
    Bg = "bg",
    El = "el",
    Tr = "tr",
    Sv = "sv",
    Da = "da",
    Nb = "nb",
    Fi = "fi",
    ZhCn = "zh-CN",
    ZhTw = "zh-TW",
    Ja = "ja",
    Ko = "ko",
    Th = "th",
    Vi = "vi",
    Id = "id",
}

impl Language {
    pub fn resolved(self) -> &'static str {
        self.tag()
            .unwrap_or_else(|| resolve_system_locale(sys_locale::get_locale().as_deref()))
    }
}

/// Maps a system locale such as `pt_BR.UTF-8`, `zh-Hant-TW` or `es-MX` to a
/// supported tag, English when nothing matches. Regional variants follow
/// Steam's own split: Spain vs Latin America, Portugal vs Brazil, simplified
/// vs traditional Chinese.
pub fn resolve_system_locale(locale: Option<&str>) -> &'static str {
    let Some(locale) = locale else {
        return "en";
    };
    let locale = locale.split(['.', '@']).next().unwrap_or("");
    let parts: Vec<String> = locale
        .split(['-', '_'])
        .map(|part| part.to_ascii_lowercase())
        .collect();
    let language = parts.first().map(String::as_str).unwrap_or("");
    let has = |value: &str| parts[1..].iter().any(|part| part == value);
    let tag = match language {
        "es" => {
            if parts.len() == 1 || has("es") {
                "es-ES"
            } else {
                "es-419"
            }
        }
        "pt" => {
            if has("br") {
                "pt-BR"
            } else {
                "pt-PT"
            }
        }
        "zh" => {
            if has("tw") || has("hk") || has("mo") || has("hant") {
                "zh-TW"
            } else {
                "zh-CN"
            }
        }
        "nb" | "nn" | "no" => "nb",
        "in" => "id",
        other => other,
    };
    Language::SUPPORTED
        .iter()
        .copied()
        .find(|supported| supported.eq_ignore_ascii_case(tag))
        .unwrap_or("en")
}
