import { InstallKind } from "../accounts/installations.constants";
import { t } from "../i18n/i18n.svelte";

const installationKindLabels: Record<InstallKind, string | null> = {
  [InstallKind.NATIVE]: null,
  [InstallKind.FLATPAK]: "Flatpak",
  [InstallKind.SNAP]: "Snap",
  [InstallKind.WINDOWS]: "Windows",
};

export const installationLabel = (entry: { kind: InstallKind; root: string }): string =>
  `${installationKindLabels[entry.kind] ?? t.installation_native()}: ${entry.root}`;
