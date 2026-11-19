import type { Language } from "../bindings/Language";

export type ResolvedLanguageType = Exclude<Language, "auto">;

export const RESOLVED_LANGUAGES: readonly ResolvedLanguageType[] = [
  "en",
  "fr",
  "de",
  "es-ES",
  "es-419",
  "pt-PT",
  "pt-BR",
  "it",
  "nl",
  "pl",
  "ru",
  "uk",
  "cs",
  "hu",
  "ro",
  "bg",
  "el",
  "tr",
  "sv",
  "da",
  "nb",
  "fi",
  "zh-CN",
  "zh-TW",
  "ja",
  "ko",
  "th",
  "vi",
  "id",
];

export const LANGUAGE_LABELS: Record<ResolvedLanguageType, string> = {
  en: "English",
  fr: "Français",
  de: "Deutsch",
  "es-ES": "Español (España)",
  "es-419": "Español (Latinoamérica)",
  "pt-PT": "Português (Portugal)",
  "pt-BR": "Português (Brasil)",
  it: "Italiano",
  nl: "Nederlands",
  pl: "Polski",
  ru: "Русский",
  uk: "Українська",
  cs: "Čeština",
  hu: "Magyar",
  ro: "Română",
  bg: "Български",
  el: "Ελληνικά",
  tr: "Türkçe",
  sv: "Svenska",
  da: "Dansk",
  nb: "Norsk",
  fi: "Suomi",
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
  ja: "日本語",
  ko: "한국어",
  th: "ไทย",
  vi: "Tiếng Việt",
  id: "Bahasa Indonesia",
};

// ! Same rules as `resolve_system_locale` on the Rust side; English when nothing matches.
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
  return "en";
};
