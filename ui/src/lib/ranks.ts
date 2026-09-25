/* Rank display: icon, Premier colours, rating formatting. */

import { hasRankIcon } from "./rank-icons";
import type { Account } from "./types";

/** Premier rating bands, sampled from the in-game tier badges. */
export const PREMIER_BANDS = [
  { min: 0, max: 4999, name: "grey", colour: "#797e86" },
  { min: 5000, max: 9999, name: "light blue", colour: "#5ab4f0" },
  { min: 10000, max: 14999, name: "blue", colour: "#3c62f0" },
  { min: 15000, max: 19999, name: "purple", colour: "#8f51f0" },
  { min: 20000, max: 24999, name: "pink", colour: "#dc48a6" },
  { min: 25000, max: 29999, name: "red", colour: "#dd4241" },
  { min: 30000, max: 50000, name: "gold", colour: "#e6be3f" },
];

export const UNRANKED = "Unranked";

/** Icon URL for a rank, or `null` when that ladder has no artwork for it. */
export function rankIconUrl(template: string, family: string): string | null {
  return hasRankIcon(template, family) ? `/ranks/${template}/${family}.png` : null;
}

/** Rating band of a Premier number, falling back to the lowest band. */
export function premierBand(rating: number | string) {
  const value = Number(rating);
  if (!Number.isFinite(value)) {
    return PREMIER_BANDS[0];
  }
  return (
    PREMIER_BANDS.find((band) => value >= band.min && value <= band.max) ??
    PREMIER_BANDS[PREMIER_BANDS.length - 1]
  );
}

/** Ratings read better with a thin space every three digits. */
export function formatRating(value: unknown): string {
  const digits = String(value ?? "").replace(/[^0-9]/g, "");
  if (!digits) {
    return "";
  }
  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

export interface RankBadge {
  text: string;
  colour: string;
  icon: string | null;
}

/**
 * How to draw an account rank: `text`, an optional `icon`, and the `colour` to
 * use for it (empty for tiered ranks, which rely on their icon).
 */
export function rankBadge(entry: Account): RankBadge {
  const value = entry.rank || UNRANKED;
  if (value === UNRANKED) {
    return { text: "", colour: "", icon: null };
  }

  if (entry.rank_kind === "rating") {
    return {
      text: formatRating(value),
      colour: premierBand(value).colour,
      icon: null,
    };
  }

  return {
    text: value,
    colour: "",
    icon: rankIconUrl(entry.rank_template, entry.rank_family),
  };
}
