export enum Language {
  AUTO = "auto",
  EN = "en",
  FR = "fr",
}

export type ResolvedLanguageType = Exclude<Language, Language.AUTO>;
