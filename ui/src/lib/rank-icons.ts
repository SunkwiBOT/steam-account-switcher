export const RANK_ICONS: Record<string, string[]> = {
  cs2: [
    "distinguished-master-guardian",
    "global-elite",
    "gold-nova-1",
    "gold-nova-2",
    "gold-nova-3",
    "gold-nova-master",
    "legendary-eagle",
    "legendary-eagle-master",
    "master-guardian-1",
    "master-guardian-2",
    "master-guardian-elite",
    "silver-1",
    "silver-2",
    "silver-3",
    "silver-4",
    "silver-elite",
    "silver-elite-master",
    "supreme-master-first-class",
  ],
  "marvel-rivals": [
    "bronze",
    "celestial",
    "diamond",
    "eternity",
    "gold",
    "grandmaster",
    "one-above-all",
    "platinum",
    "silver",
  ],
  overwatch: [
    "bronze",
    "diamond",
    "emerald",
    "gold",
    "grandmaster",
    "platinum",
    "silver",
    "top-500",
  ],
};

export function hasRankIcon(template: string, family: string): boolean {
  return Boolean(family) && (RANK_ICONS[template] ?? []).includes(family);
}
