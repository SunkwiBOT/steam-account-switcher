import { mount } from "svelte";

import "./app.css";

import App from "./components/layout/App.svelte";
import { startBridge } from "./lib/api/bridge";

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

  mount(App, {
    target: document.getElementById("app")!,
  });

  await startBridge();
};

bootstrap().catch((error) => console.error(error));
