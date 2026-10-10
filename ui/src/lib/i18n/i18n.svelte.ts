import { m } from "../paraglide/messages.js";
import { setLocale } from "../paraglide/runtime.js";
import { Language } from "./i18n.constants";
import type { ResolvedLanguageType } from "./i18n.constants";
import type { MessagePartType } from "./messages";

const systemLocale = (): ResolvedLanguageType =>
  (navigator.languages[0] ?? navigator.language).toLowerCase().startsWith("fr")
    ? Language.FR
    : Language.EN;

export const locale = $state<{ preference: Language; current: ResolvedLanguageType }>({
  preference: Language.AUTO,
  current: systemLocale(),
});

export const applyLanguage = (preference: Language, resolved?: ResolvedLanguageType): void => {
  locale.preference = preference;
  locale.current = resolved ?? (preference === Language.AUTO ? systemLocale() : preference);
  setLocale(locale.current, { reload: false });
  document.documentElement.lang = locale.current;
};

export const t: typeof m = new Proxy(m, {
  get(target, key: keyof typeof m) {
    return (
      inputs: Record<string, unknown> = {},
      options: { locale?: ResolvedLanguageType } = {},
    ): string =>
      Reflect.apply(target[key], undefined, [inputs, { locale: locale.current, ...options }]);
  },
});

export const rich = <Inputs extends Record<string, string | number>>(
  translate: (inputs: Inputs) => string,
  values: Inputs,
  codeKeys: readonly string[] = [],
): MessagePartType[] => {
  const entries = Object.entries(values);
  const markers = Object.fromEntries(entries.map(([key], index) => [key, `\u0000${index}\u0000`]));
  const rendered: string = Reflect.apply(translate, undefined, [markers]);
  return rendered.split("\u0000").flatMap((text, index) => {
    if (index % 2 === 1) {
      const [key, value] = entries[Number(text)];
      return [
        { text: String(value), bold: true, ...(codeKeys.includes(key) ? { code: true } : {}) },
      ];
    }
    return text ? [{ text }] : [];
  });
};

export const folderLabel = (folder: { system: boolean; name: string }): string =>
  folder.system ? t.guard_saved_folder() : folder.name;

export const formatDate = (timestamp: number | string, time = true): string => {
  if (timestamp === 0) return t.common_unavailable();
  const date =
    typeof timestamp === "number" ? new Date(timestamp * 1000) : new Date(`${timestamp}T12:00:00`);
  if (!Number.isFinite(date.getTime())) return "—";
  return new Intl.DateTimeFormat(locale.current, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    ...(time ? ({ hour: "2-digit", minute: "2-digit", second: "2-digit" } as const) : {}),
  }).format(date);
};

export const relativeDate = (timestamp: number): string => {
  if (!timestamp) return t.common_unknown();
  const seconds = Math.round((timestamp * 1000 - Date.now()) / 1000);
  const ranges: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31536000],
    ["month", 2592000],
    ["day", 86400],
    ["hour", 3600],
    ["minute", 60],
    ["second", 1],
  ];
  const [unit, divisor] =
    ranges.find(([, size]) => Math.abs(seconds) >= size) ?? ranges[ranges.length - 1];
  return new Intl.RelativeTimeFormat(locale.current, { numeric: "auto" }).format(
    Math.round(seconds / divisor),
    unit,
  );
};

applyLanguage(Language.AUTO);
