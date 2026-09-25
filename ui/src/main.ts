import { mount } from "svelte";

import "./app.css";
import App from "./components/App.svelte";
import { startBridge } from "./lib/bridge";

/** Hides the context menu WebKitGTK shows on its own: the application is a
 *  window, not a page, and right clicking an account or a folder opens the
 *  actions menu instead. Fields and selected text keep it, so the mouse still
 *  copies and pastes like everywhere else. */
function suppressDefaultContextMenu(): void {
  window.addEventListener("contextmenu", (event) => {
    const editable =
      event.target instanceof Element && event.target.closest("input, textarea, [contenteditable]");
    const selected = window.getSelection()?.toString() ?? "";
    if (editable || selected) {
      return;
    }
    event.preventDefault();
  });
}

async function bootstrap(): Promise<void> {
  suppressDefaultContextMenu();
  mount(App, { target: document.getElementById("app")! });
  await startBridge();
}

bootstrap().catch((error) => console.error(error));
