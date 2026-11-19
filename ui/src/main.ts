import { mount } from "svelte";

import "@fontsource-variable/inter";
import "./app.css";

import App from "./components/layout/App.svelte";
import ToastCard from "./components/layout/ToastCard.svelte";
import { startBridge } from "./lib/api/bridge";
import { setToastCard } from "./lib/state/toasts.svelte";

setToastCard(ToastCard);

const suppressDefaultContextMenu = (): void => {
  window.addEventListener("contextmenu", (event) => {
    const editable =
      event.target instanceof Element && event.target.closest("input, textarea, [contenteditable]");
    const selected = window.getSelection()?.toString() ?? "";

    if (editable || selected) {
      return;
    }

    event.preventDefault();
  });
};

const bootstrap = async (): Promise<void> => {
  suppressDefaultContextMenu();

  const target = document.getElementById("app");
  if (target) {
    mount(App, { target });
  }

  await startBridge();
};

bootstrap().catch((error) => console.error(error));
