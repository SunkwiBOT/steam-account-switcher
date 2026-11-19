export type VaultCountdownType =
  | { hours: number; minutes: string; seconds: string }
  | { minutes: number; seconds: string }
  | { seconds: number };

const pad = (value: number): string => String(value).padStart(2, "0");

export const vaultCountdown = (seconds: number): VaultCountdownType => {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remainder = total % 60;
  if (hours > 0) {
    return { hours, minutes: pad(minutes), seconds: pad(remainder) };
  }
  if (minutes > 0) {
    return { minutes, seconds: pad(remainder) };
  }
  return { seconds: remainder };
};
