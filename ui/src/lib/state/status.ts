import type { InstallKind } from "../bindings/InstallKind";
import { t } from "../i18n/i18n.svelte";

const installationKindLabels: Record<InstallKind, string | null> = {
  NATIVE: null,
  FLATPAK: "Flatpak",
  SNAP: "Snap",
  WINDOWS: "Windows",
};

export const installationLabel = (entry: { kind: InstallKind; root: string }): string =>
  `${installationKindLabels[entry.kind] ?? t.installation_native()}: ${entry.root}`;
