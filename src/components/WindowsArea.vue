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
  <div class="flex items-center justify-center flex-grow overflow-x-auto overflow-y-hidden scrollbar-thin scrollbar-thumb-white/20 scrollbar-track-transparent">
    <TransitionGroup 
      name="window-list"
      tag="div"
      class="flex items-center justify-center gap-0.5"
    >
      <WindowButton
        v-for="window in windows"
        :key="window.id"
        v-bind="window"
      />
    </TransitionGroup>
  </div>
</template>

<style >
.window-list-move {
  transition: transform 0.3s ease;
}
.window-list-enter-active,
.window-list-leave-active {
  transition: all 0.3s ease;
}
.window-list-enter-from,
.window-list-leave-to {
  opacity: 0;
  transform: translateY(30px);
}
</style>
