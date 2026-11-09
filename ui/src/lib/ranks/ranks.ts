import { t } from "../i18n/i18n.svelte";

import { hasRankIcon } from "./rank-icons";
import { RankKind } from "./ranks.constants";
import type { AccountType } from "../types";

/** Premier rating bands, sampled from the in-game tier badges. */
export const PREMIER_BANDS = [
  {
    min: 0,
    max: 4999,
    name: "grey",
    colour: "#797e86",
  },
  {
    min: 5000,
    max: 9999,
    name: "light blue",
    colour: "#5ab4f0",
  },
  {
    min: 10000,
    max: 14999,
    name: "blue",
    colour: "#3c62f0",
  },
  {
    min: 15000,
    max: 19999,
    name: "purple",
    colour: "#8f51f0",
  },
  {
    min: 20000,
    max: 24999,
    name: "pink",
    colour: "#dc48a6",
  },
  {
    min: 25000,
    max: 29999,
    name: "red",
    colour: "#dd4241",
  },
  {
    min: 30000,
    max: 50000,
    name: "gold",
    colour: "#e6be3f",
  },
];

export const UNRANKED = "Unranked";

const rankNames: Record<string, () => string> = {
  Bronze: t.rank_bronze,
  Silver: t.rank_silver,
  Gold: t.rank_gold,
  Platinum: t.rank_platinum,
  Diamond: t.rank_diamond,
  Grandmaster: t.rank_grandmaster,
  Celestial: t.rank_celestial,
  Emerald: t.rank_emerald,
  Eternity: t.rank_eternity,
  "Top 500": t.rank_top_500,
  "Silver I": t.rank_silver_i,
  "Silver II": t.rank_silver_ii,
  "Silver III": t.rank_silver_iii,
  "Silver IV": t.rank_silver_iv,
  "Silver Elite": t.rank_silver_elite,
  "Silver Elite Master": t.rank_silver_elite_master,
  "Gold Nova I": t.rank_gold_nova_i,
  "Gold Nova II": t.rank_gold_nova_ii,
  "Gold Nova III": t.rank_gold_nova_iii,
  "Gold Nova Master": t.rank_gold_nova_master,
  "Master Guardian I": t.rank_master_guardian_i,
  "Master Guardian II": t.rank_master_guardian_ii,
  "Master Guardian Elite": t.rank_master_guardian_elite,
  "Distinguished Master Guardian": t.rank_distinguished_master_guardian,
  "Legendary Eagle": t.rank_legendary_eagle,
  "Legendary Eagle Master": t.rank_legendary_eagle_master,
  "Supreme Master First Class": t.rank_supreme_master_first_class,
  "Global Elite": t.rank_global_elite,
};

export const rankLabel = (value: string): string => {
  if (!value || value === UNRANKED) return t.rank_unranked();
  if (rankNames[value]) return rankNames[value]();
  const match = /^(.+) ([1-5])$/.exec(value);
  return match && rankNames[match[1]]
    ? t.rank_division({ rank: rankNames[match[1]](), division: match[2] })
    : value;
};

// ! An account card has room for about ten characters beside the playtime and the icon: the
// long names get the short forms players use; the full name stays in the tooltip.
const SHORT_RANKS: Record<string, string> = {
  Grandmaster: "GM",
  "Silver I": "S1",
  "Silver II": "S2",
  "Silver III": "S3",
  "Silver IV": "S4",
  "Silver Elite": "SE",
  "Silver Elite Master": "SEM",
  "Gold Nova I": "GN1",
  "Gold Nova II": "GN2",
  "Gold Nova III": "GN3",
  "Gold Nova Master": "GNM",
  "Master Guardian I": "MG1",
  "Master Guardian II": "MG2",
  "Master Guardian Elite": "MGE",
  "Distinguished Master Guardian": "DMG",
  "Legendary Eagle": "LE",
  "Legendary Eagle Master": "LEM",
  "Supreme Master First Class": "SMFC",
  "Global Elite": "GE",
};

const rankShortLabel = (value: string): string => {
  if (SHORT_RANKS[value]) return SHORT_RANKS[value];
  const match = /^(.+) ([1-5])$/.exec(value);
  return match && SHORT_RANKS[match[1]] ? `${SHORT_RANKS[match[1]]}${match[2]}` : rankLabel(value);
};

const colourLabel = (name: string): string =>
  (
    ({
      grey: t.colour_grey,
      "light blue": t.colour_light_blue,
      blue: t.colour_blue,
      purple: t.colour_purple,
      pink: t.colour_pink,
      red: t.colour_red,
      gold: t.colour_gold,
    })[name] ?? t.common_unknown
  )();

export const rankIconUrl = (template: string, family: string): string | null => {
  return hasRankIcon(template, family) ? `/ranks/${template}/${family}.png` : null;
};

export const premierBand = (rating: number | string) => {
  const value = Number(rating);
  if (!Number.isFinite(value)) {
    return { ...PREMIER_BANDS[0], name: colourLabel(PREMIER_BANDS[0].name) };
  }
  const band =
    PREMIER_BANDS.find((band) => value >= band.min && value <= band.max) ??
    PREMIER_BANDS[PREMIER_BANDS.length - 1];
  return { ...band, name: colourLabel(band.name) };
};

export const formatRating = (value: unknown): string => {
  const digits = String(value ?? "").replace(/[^0-9]/g, "");
  if (!digits) {
    return "";
  }
  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, " ");
};

export type RankBadgeType = {
  text: string;
  short: string;
  colour: string;
  icon: string | null;
};

export const rankBadge = (entry: AccountType): RankBadgeType => {
  const value = entry.rank || UNRANKED;
  if (value === UNRANKED) {
    return { text: "", short: "", colour: "", icon: null };
  }

  if (entry.rank_kind === RankKind.RATING) {
    return {
      text: formatRating(value),
      short: formatRating(value),
      colour: premierBand(value).colour,
      icon: null,
    };
  }

  return {
    text: rankLabel(value),
    short: rankShortLabel(value),
    colour: "",
    icon: rankIconUrl(entry.rank_template, entry.rank_family),
  };
};
