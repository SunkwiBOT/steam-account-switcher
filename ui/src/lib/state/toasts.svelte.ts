import type { Component } from "svelte";
import { toast as sonner } from "svelte-sonner";

import type { MessageType } from "../i18n/messages";
import type { ToastAccountType, ToastFolderType } from "../types";
import { TOAST_DURATION_MS, type ToastKind } from "./state.constants";

export type ToastCardPropsType = {
  kind: ToastKind;
  title: string;
  description: MessageType | null;
  account: ToastAccountType | null;
  folder: ToastFolderType | null;
  onclose: () => void;
};

export type ToastOptionsType = {
  description?: MessageType;
  account?: ToastAccountType | null;
  folder?: ToastFolderType | null;
};

let toastId = 0;

let card: Component<ToastCardPropsType>;

export const setToastCard = (component: Component<ToastCardPropsType>): void => {
  card = component;
};

export const dismissToast = (id: string): void => {
  sonner.dismiss(id);
};

export const showToast = (kind: ToastKind, title: string, options: ToastOptionsType = {}): void => {
  toastId += 1;
  const id = `toast-${toastId}`;

  sonner.custom(card, {
    id,
    duration: TOAST_DURATION_MS[kind],
    componentProps: {
      kind,
      title,
      description: options.description ?? null,
      account: options.account ?? null,
      folder: options.folder ?? null,
      onclose: () => dismissToast(id),
    },
  });
};
