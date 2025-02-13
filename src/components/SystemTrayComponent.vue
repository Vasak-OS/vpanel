<script lang="ts" setup>
import ClockComponent from "./ClockComponent.vue";
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface TrayIcon {
  id: string;
  wid: number;
  icon_data?: string;
}

const trayIcons = ref<TrayIcon[]>([]);
let unlisten: (() => void) | null = null;

const refreshTrayIcons = async () => {
  try {
    trayIcons.value = await invoke("get_tray_items");
    console.info("Tray icons:", trayIcons.value);
  } catch (error) {
    console.error("Error fetching tray icons:", error);
  }
};

onMounted(async () => {
  await refreshTrayIcons();

  // Listen for tray updates from Rust
  unlisten = await listen("tray-update", () => {
    refreshTrayIcons();
  });
});

onUnmounted(async () => {
  if (unlisten) {
    unlisten();
  }
});
</script>

<template>
  <div class="navale-notification-area">
    <div
      v-for="icon in trayIcons"
      :key="icon.id"
      class="w-6 h-6 flex items-center justify-center hover:bg-gray-700 rounded cursor-pointer"
    >
      <img
        v-if="icon.icon_data"
        :src="`data:image/png;base64,${icon.icon_data}`"
        class="w-4 h-4"
        alt=""
      />
      <div v-else class="w-4 h-4 bg-gray-500 rounded-full"></div>
    </div>
    <ClockComponent />
  </div>
</template>
