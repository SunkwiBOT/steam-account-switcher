export type VaultEditorFieldsType = {
  currentPassword: string;
  newPassword: string;
  repeated: string;
};

export const vaultEditorDirty = (
  fields: VaultEditorFieldsType,
  options: { disabling: boolean; requiresCurrent: boolean },
): boolean =>
  options.disabling
    ? fields.currentPassword !== ""
    : (options.requiresCurrent && fields.currentPassword !== "") ||
      fields.newPassword !== "" ||
      fields.repeated !== "";
