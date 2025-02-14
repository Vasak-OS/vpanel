<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface TrayItem {
  id: string;
  wid: number;
  icon_data?: string;
  title?: string;
  menu?: TrayMenu[];
}

interface TrayMenu {
  id: string;
  label: string;
  enabled: boolean;
  checked?: boolean;
}

const trayItems = ref<TrayItem[]>([]);
let unlisten: (() => void) | null = null;

const refreshTrayItems = async (): Promise<void> => {
  try {
    trayItems.value = await invoke("get_tray_items");
  } catch (error) {
    console.error("[TrayPanel Error] Error obteniendo items:", error);
  }
};

const handleTrayClick = async (item: TrayItem, event: MouseEvent) => {
  try {
    await invoke("handle_tray_click", { 
      id: item.id,
      button: event.button,
      x: event.clientX,
      y: event.clientY 
    });
  } catch (error) {
    console.error("[TrayPanel Error] Error manejando click:", error);
  }
};

onMounted(async () => {
  await refreshTrayItems();
  unlisten = await listen("tray-update", refreshTrayItems);
  console.log(trayItems.value);
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="tray-panel">
    <div
      v-for="item in trayItems"
      :key="item.id"
      class="tray-item"
      @click="(e) => handleTrayClick(item, e)"
      @contextmenu.prevent="(e) => handleTrayClick(item, e)"
    >
      <img
        v-if="item.icon_data"
        :src="`data:image/png;base64,${item.icon_data}`"
        :alt="item.title || ''"
        class="tray-icon"
      />
      <div v-else class="icon-placeholder" />
      <span v-if="item.title" class="tray-title">{{ item.title }}</span>
    </div>
  </div>
</template>

<style scoped>
.tray-panel {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 8px;
  height: 100%;
}

.tray-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.tray-item:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.tray-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
}

.icon-placeholder {
  width: 16px;
  height: 16px;
  background-color: rgba(128, 128, 128, 0.5);
  border-radius: 4px;
}

.tray-title {
  font-size: 12px;
  color: #ffffff;
}
</style> 