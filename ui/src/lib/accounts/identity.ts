import { app } from "../state/state.svelte";

/// Sign-in names are hidden by default, so an account without a known persona
/// must never show its login name; the interface blurs it instead.
export const hiddenIdentity = (entry: { persona_name: string }): boolean =>
  Boolean(app.snapshot?.masked) && !entry.persona_name;
