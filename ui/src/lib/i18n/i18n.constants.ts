//! Tags mirror `Language` in src-tauri/src/locale.rs, the Paraglide locales
//! and the rust-i18n files: the lists must stay identical.
export enum Language {
  AUTO = "auto",
  EN = "en",
  FR = "fr",
  DE = "de",
  ESES = "es-ES",
  ES419 = "es-419",
  PTPT = "pt-PT",
  PTBR = "pt-BR",
  IT = "it",
  NL = "nl",
  PL = "pl",
  RU = "ru",
  UK = "uk",
  CS = "cs",
  HU = "hu",
  RO = "ro",
  BG = "bg",
  EL = "el",
  TR = "tr",
  SV = "sv",
  DA = "da",
  NB = "nb",
  FI = "fi",
  ZHCN = "zh-CN",
  ZHTW = "zh-TW",
  JA = "ja",
  KO = "ko",
  TH = "th",
  VI = "vi",
  ID = "id",
}

export type ResolvedLanguageType = Exclude<Language, Language.AUTO>;

export const RESOLVED_LANGUAGES: readonly ResolvedLanguageType[] = Object.values(Language).filter(
  (value): value is ResolvedLanguageType => value !== Language.AUTO,
);

/** Each language named in itself, so the picker reads the same in every language. */
export const LANGUAGE_LABELS: Record<ResolvedLanguageType, string> = {
  [Language.EN]: "English",
  [Language.FR]: "Français",
  [Language.DE]: "Deutsch",
  [Language.ESES]: "Español (España)",
  [Language.ES419]: "Español (Latinoamérica)",
  [Language.PTPT]: "Português (Portugal)",
  [Language.PTBR]: "Português (Brasil)",
  [Language.IT]: "Italiano",
  [Language.NL]: "Nederlands",
  [Language.PL]: "Polski",
  [Language.RU]: "Русский",
  [Language.UK]: "Українська",
  [Language.CS]: "Čeština",
  [Language.HU]: "Magyar",
  [Language.RO]: "Română",
  [Language.BG]: "Български",
  [Language.EL]: "Ελληνικά",
  [Language.TR]: "Türkçe",
  [Language.SV]: "Svenska",
  [Language.DA]: "Dansk",
  [Language.NB]: "Norsk",
  [Language.FI]: "Suomi",
  [Language.ZHCN]: "简体中文",
  [Language.ZHTW]: "繁體中文",
  [Language.JA]: "日本語",
  [Language.KO]: "한국어",
  [Language.TH]: "ไทย",
  [Language.VI]: "Tiếng Việt",
  [Language.ID]: "Bahasa Indonesia",
};

/** Same rules as `resolve_system_locale` on the Rust side; English when nothing matches. */
export const resolveSystemLocale = (locales: readonly string[]): ResolvedLanguageType => {
  for (const raw of locales) {
    const parts = raw
      .split(/[.@]/)[0]
      .split(/[-_]/)
      .map((part) => part.toLowerCase());
    const language = parts[0] ?? "";
    const has = (value: string): boolean => parts.slice(1).includes(value);
    let tag = language;
    if (language === "es") tag = parts.length === 1 || has("es") ? "es-es" : "es-419";
    else if (language === "pt") tag = has("br") ? "pt-br" : "pt-pt";
    else if (language === "zh")
      tag = has("tw") || has("hk") || has("mo") || has("hant") ? "zh-tw" : "zh-cn";
    else if (language === "nb" || language === "nn" || language === "no") tag = "nb";
    else if (language === "in") tag = "id";
    const match = RESOLVED_LANGUAGES.find((candidate) => candidate.toLowerCase() === tag);
    if (match) return match;
  }
  return Language.EN;
};
