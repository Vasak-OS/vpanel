<script lang="ts" setup>
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import WindowButtom from "./buttons/WindowButtom.vue";

interface WindowInfo {
  id: string;
  title: string;
  is_minimized: boolean;
  icon: string;
}

const windows = ref<WindowInfo[]>([]);
let unlisten: (() => void) | null = null;

const refreshWindows = async () => {
  try {
    windows.value = await invoke("get_windows");
  } catch (error) {
    console.error("Error fetching windows:", error);
  }
};

onMounted(async () => {
  await refreshWindows();
  // Listen for window updates from Rust
  unlisten = await listen("window-update", () => {
    refreshWindows();
  });
});
onUnmounted(async () => {
  if (unlisten) {
    unlisten();
  }
});
</script>
<template>
  <div class="flex">
    <WindowButtom
      v-for="window in windows"
      :key="window.id"
      :id="window.id"
      :title="window.title"
      :is_minimized="window.is_minimized"
      :icon="window.icon"
    />
  </div>
</template>
