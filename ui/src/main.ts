import { mount } from "svelte";

import "./app.css";
import App from "./components/App.svelte";
import { startBridge } from "./lib/bridge";

async function bootstrap(): Promise<void> {
  mount(App, { target: document.getElementById("app")! });
  await startBridge();
}

bootstrap().catch((error) => console.error(error));
