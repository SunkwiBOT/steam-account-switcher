import { t } from "../i18n/i18n.svelte";
export type SteamIdentifierType = {
  label: string;
  value: string;
};

/** Public individual Steam IDs only. BigInt preserves all 64 bits. */
export const steamIdentifiers = (steamId: string): SteamIdentifierType[] => {
  if (!/^[0-9]{1,20}$/.test(steamId)) {
    return [];
  }

  const id = BigInt(steamId);
  const accountId = id & BigInt(0xffffffff);

  if (
    id >> BigInt(56) !== BigInt(1) ||
    ((id >> BigInt(52)) & BigInt(15)) !== BigInt(1) ||
    ((id >> BigInt(32)) & BigInt(0xfffff)) !== BigInt(1) ||
    accountId === BigInt(0)
  ) {
    return [];
  }

  // Steam's invite alphabet maps the hexadecimal account ID, split at its midpoint.
  // Format reference: ValvePython/steam, steam/steamid.py (as_invite_code).
  const hexAlphabet = "0123456789abcdef";
  const inviteAlphabet = "bcdfghjkmnpqrtvw";
  const invite = [...accountId.toString(16)]
    .map((digit) => inviteAlphabet[hexAlphabet.indexOf(digit)])
    .join("");
  const midpoint = Math.floor(invite.length / 2);
  const code = midpoint ? `${invite.slice(0, midpoint)}-${invite.slice(midpoint)}` : invite;

  return [
    {
      label: t.steam_ids_64(),
      value: id.toString(),
    },
    {
      label: t.steam_ids_account(),
      value: accountId.toString(),
    },
    {
      label: t.steam_ids_steam2(),
      value: `STEAM_1:${accountId % BigInt(2)}:${accountId / BigInt(2)}`,
    },
    {
      label: t.steam_ids_steam3(),
      value: `[U:1:${accountId}]`,
    },
    {
      label: t.steam_ids_profile_url(),
      value: `https://steamcommunity.com/profiles/${id}`,
    },
    {
      label: t.steam_ids_invite_url(),
      value: `https://steamcommunity.com/user/${code}`,
    },
    {
      label: t.steam_ids_short_invite_url(),
      value: `https://s.team/p/${code}`,
    },
    {
      label: t.steam_ids_fivem_hex(),
      value: `steam:${id.toString(16)}`,
    },
  ];
};
