import type { AccountDto } from "./bindings/AccountDto";
import type { FolderDto } from "./bindings/FolderDto";

export type ToastAccountType = Pick<
  AccountDto,
  "avatar" | "initial" | "display_name" | "persona_name"
>;

export type ToastFolderType = Pick<FolderDto, "name" | "color" | "icon" | "system">;

export type AvatarSourceType = {
  avatar: string | null;
  initial: string;
  persona_name?: string;
};

export type TableCellType = {
  text: string;
  title?: string;
  avatar?: AvatarSourceType;
};
