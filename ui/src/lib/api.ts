/* Shared formatting helpers. */

/** Card accent colours, identical to the swatches in the account dialog. */
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

function escapeHtml(value: unknown): string {
  return String(value ?? "").replace(
    /[&<>"']/g,
    (character) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[character] as string,
  );
}

/** Escaped value in bold, for sentences that mix fixed text and a name. */
export function strong(value: unknown): string {
  return `<strong>${escapeHtml(value)}</strong>`;
}

export function formatSeconds(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  if (total < 60) {
    return total === 0 ? "0m" : "<1m";
  }
  const minutes = Math.floor(total / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) {
    return `${rest}m`;
  }
  return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`;
}
