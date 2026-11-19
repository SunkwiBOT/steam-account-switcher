import type { SearchScope } from "../bindings/SearchScope";
import type { AccountDto } from "../bindings/AccountDto";

export const matchesSearch = (entry: AccountDto, needle: string, scope: SearchScope): boolean => {
  const names =
    entry.display_name.toLowerCase().includes(needle) ||
    entry.persona_name.toLowerCase().includes(needle);
  if (scope === "DISPLAY_NAME") return names;
  const identifiers =
    entry.account_name.toLowerCase().includes(needle) || entry.steam_id.includes(needle);
  return scope === "IDENTIFIER" ? identifiers : names || identifiers;
};
