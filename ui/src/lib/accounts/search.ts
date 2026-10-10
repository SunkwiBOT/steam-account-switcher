import { SearchScope } from "../state/state.constants";
import type { AccountType } from "../types";

/**
 * Matches an account against an already lowercased needle. `display_name` only
 * looks at the names shown on the card, `identifier` only at the Steam login and
 * the Steam ID, and `both` at all of them.
 */
export const matchesSearch = (entry: AccountType, needle: string, scope: SearchScope): boolean => {
  const names =
    entry.display_name.toLowerCase().includes(needle) ||
    entry.persona_name.toLowerCase().includes(needle);
  if (scope === SearchScope.DISPLAY_NAME) return names;
  const identifiers =
    entry.account_name.toLowerCase().includes(needle) || entry.steam_id.includes(needle);
  return scope === SearchScope.IDENTIFIER ? identifiers : names || identifiers;
};
