import { errorMessage } from "../errors/errors.svelte";
import { t } from "../i18n/i18n.svelte";
import { matchesSearch } from "../accounts/search";
import type { AccountDto } from "../bindings/AccountDto";
import { accounts, app } from "./state.svelte";

export const visibleAccounts = (): AccountDto[] => {
  const needle = app.search.trim().toLowerCase();
  const scope = app.snapshot?.settings.search_scope ?? "BOTH";
  return accounts()
    .filter((entry) => {
      if (app.folderId !== null && entry.folder_id !== app.folderId) return false;
      if (!needle) {
        return app.folderId === null ? !entry.folder_id : true;
      }
      return matchesSearch(entry, needle, scope);
    })
    .sort((left, right) => right.last_login_seconds - left.last_login_seconds);
};

export const emptyMessage = (): string => {
  const snapshot = app.snapshot;
  if (!snapshot) {
    return "";
  }
  if (snapshot.installations.length === 0) {
    return t.accounts_no_installation();
  }
  if (snapshot.accounts.length === 0) {
    if (snapshot.accounts_error) {
      return errorMessage(snapshot.accounts_error);
    }
    return t.accounts_empty();
  }
  return t.accounts_search_empty();
};
