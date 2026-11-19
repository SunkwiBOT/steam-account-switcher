import { app } from "../state/state.svelte";

export const hiddenIdentity = (entry: { persona_name: string }): boolean =>
  Boolean(app.snapshot?.masked) && !entry.persona_name;

// ! Matches the backend's `MASKED_LABEL` (steam-core's `model.rs`).
export const MASKED_LABEL = "******";
