<script lang="ts" setup>
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import WindowButton from "./buttons/WindowButton.vue";

interface WindowInfo {
  id: string;
  title: string;
  is_minimized: boolean;
  icon: string;
}

const windows = ref<WindowInfo[]>([]);
let unlisten: (() => void) | null = null;

const refreshWindows = async (): Promise<void> => {
  try {
    windows.value = await invoke("get_windows");
  } catch (error) {
    console.error("[Windows Error] Error obteniendo ventanas:", error);
  }
};

onMounted(async () => {
  await refreshWindows();
  unlisten = await listen("window-update", refreshWindows);
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="flex items-center justify-center gap-0.5 flex-grow mx-2">
    <WindowButton
      v-for="window in windows"
      :key="window.id"
      v-bind="window"
    />
  </div>
</template>

<style scoped>
.windows-container {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 2px;
  flex-grow: 1;
  margin: 0 8px;
}
</style>
