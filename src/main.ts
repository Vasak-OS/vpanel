import { createApp } from "vue";
import { setPanelProperties } from "./common/window";
import App from "./App.vue";
import { Position } from "@tauri-apps/plugin-positioner";

const app = createApp(App);
setPanelProperties(Position.BottomRight);
app.mount("#app");
