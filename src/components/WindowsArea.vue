<script lang="ts" setup>
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface WindowInfo {
  id: string;
  title: string;
  is_minimized: boolean;
  icon?: string;
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

const focusWindow = async (windowId: string) => {
  try {
    await invoke("focus_window", { windowId });
    await refreshWindows();
  } catch (error) {
    console.error("Error focusing window:", error);
  }
};

const minimizeWindow = async (windowId: string) => {
  try {
    await invoke("minimize_window", { windowId });
    await refreshWindows();
  } catch (error) {
    console.error("Error minimizing window:", error);
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
    <div
      v-for="window in windows"
      :key="window.id"
      class="px-3 py-1 hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2"
      :class="{ 'opacity-50': window.is_minimized }"
      @click="focusWindow(window.id)"
    >
      <img v-if="window.icon" :src="`data:image/jpeg;charset=utf-8;base64,${window.icon}`" class="w-4 h-4 icon" alt="" />
      <span>{{ window.title }}</span>
    </div>
  </div>
</template>

<style>
.icon{
  width: 26px;
  height: 26px;
}
</style>