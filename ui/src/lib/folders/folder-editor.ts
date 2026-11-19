export type FolderDraftType = { name: string; color: string; icon: string };
export type FolderBaselineType = { name: string; color: string | null; icon: string };

export const folderDraftDirty = (draft: FolderDraftType, baseline: FolderBaselineType): boolean =>
  draft.name !== baseline.name ||
  draft.color !== (baseline.color ?? "") ||
  draft.icon !== baseline.icon;
