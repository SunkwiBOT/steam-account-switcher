import { t } from "../i18n/i18n.svelte";

export const CARD_COLORS: string[] = [
  "",
  "#3b82f6",
  "#06b6d4",
  "#10b981",
  "#84cc16",
  "#f59e0b",
  "#f97316",
  "#f43f5e",
  "#ec4899",
  "#8b5cf6",
  "#71717a",
];

export const formatSeconds = (seconds: number): string => {
  const total = Math.max(0, Math.floor(seconds));

  if (total < 60) {
    return total === 0 ? t.duration_zero() : t.duration_less_minute();
  }

  const minutes = Math.floor(total / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;

  if (hours === 0) {
    return t.duration_minutes({
      minutes: rest,
    });
  }

  return rest === 0
    ? t.duration_hours({
        hours,
      })
    : t.duration_hours_minutes({
        hours,
        minutes: rest,
      });
};
