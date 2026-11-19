import type { FolderDto } from "../bindings/FolderDto";
import { app } from "./state.svelte";

export const folders = (): FolderDto[] => {
  const list = app.snapshot?.folders ?? [];
  return [...list.filter((folder) => folder.system), ...list.filter((folder) => !folder.system)];
};

export const currentFolder = (): FolderDto | null => {
  return folders().find((folder) => folder.id === app.folderId) ?? null;
};

export const folderById = (id: string): FolderDto | null => {
  return folders().find((folder) => folder.id === id) ?? null;
};

export const visibleFolders = (): FolderDto[] => {
  return app.search.trim() || app.folderId !== null ? [] : folders();
};
