<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface TrayItem {
  id: string;
  wid: number;
  icon_data?: string;
  title?: string;
  menu?: TrayMenu[];
  tooltip?: string;
}

interface TrayMenu {
  id: string;
  label: string;
  enabled: boolean;
  checked?: boolean;
}

const trayItems = ref<TrayItem[]>([]);

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

</script>

<template>
  <div class="flex items-center gap-1 px-2 h-full">
    <TransitionGroup 
      name="tray-list"
      tag="div"
      class="flex items-center gap-1"
    >
      <div
        v-for="item in trayItems"
        :key="item.id"
        class="group flex items-center gap-1.5 px-2 py-1 rounded-lg cursor-pointer transform transition-all duration-300 hover:bg-white/10 hover:scale-105 active:scale-95 relative"
        @click="(e) => handleTrayClick(item, e)"
        @contextmenu.prevent="(e) => handleTrayClick(item, e)"
      >
        <img
          v-if="item.icon_data"
          :src="`data:image/png;base64,${item.icon_data}`"
          :alt="item.title || ''"
          class="w-4 h-4 object-contain transition-all duration-300 group-hover:rotate-6 group-hover:brightness-110"
        />
        <div v-else class="w-4 h-4 bg-gray-500/50 rounded animate-pulse" />
        <span 
          v-if="item.title" 
          class="text-xs text-white transition-all duration-300 group-hover:text-white/90"
        >
          {{ item.title }}
        </span>
        <div class="absolute -bottom-1 opacity-0 group-hover:opacity-100 transition-all duration-300 text-xs px-2 py-0.5 bg-black/80 rounded-md whitespace-nowrap pointer-events-none transform group-hover:translate-y-5 backdrop-blur-sm">
          {{ item.tooltip || item.title }}
        </div>
      </div>
    </TransitionGroup>
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

.tray-list-move {
  transition: transform 0.3s ease;
}
.tray-list-enter-active,
.tray-list-leave-active {
  transition: all 0.3s ease;
}
.tray-list-enter-from,
.tray-list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style> 