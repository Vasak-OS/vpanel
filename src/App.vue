<script setup lang="ts">
import { onMounted, onUnmounted, ref, type Ref } from "vue";
import { Command } from '@tauri-apps/plugin-shell'
import { setPanelProperties } from "@/common/window";
import WindowsArea from '@/components/WindowsArea.vue';
import TrayPanel from '@/components/TrayPanel.vue';
import ClockComponent from '@/components/ClockComponent.vue';
import { getIconSource } from '@vasakgroup/plugin-vicons'
import { useConfigStore } from "./store/configStore";
import { listen } from "@tauri-apps/api/event";

const menuIcon: Ref<string> = ref('');
const notifyIcon: Ref<string> = ref('');
const configStore = useConfigStore();
let unlistenConfig: Function | null = null;

const setMenuIcon = async () => {
  try {
    menuIcon.value = await getIconSource('menu-editor');
  } catch (err) {
    console.error('Error: finding icon menu')
  }
}

const setNotifyIcon = async () => {
  try {
    notifyIcon.value = await getIconSource('preferences-desktop-notification')
  } catch (err) {
    console.error('Error: finding notify icon')
  }
}

const openMenu = () => {
  Command.create('vmenu').execute()
}

const openNotificationCenter = () => {
  Command.create('vasak-control-center').execute()
}

onMounted(async () => {
  setMenuIcon();
  setNotifyIcon();
  await setPanelProperties();
  configStore.loadConfig();
  unlistenConfig = await listen('config-changed', async () => {
    configStore.loadConfig();
    console.log('Config changed');
  });
});

onUnmounted(() => {
  if (unlistenConfig !== null) {
    unlistenConfig();
  }
});
</script>

<template>
  <nav
    class="flex justify-between items-center px-1 mx-1 bg-white/70 dark:bg-black/70 text-black dark:text-white h-[30px] rounded-xl backdrop-blur-md transition-all duration-300 hover:bg-white/80 hover:dark:bg-black/80">
    <img :src="menuIcon"  alt="Menu" @click="openMenu"
      class="h-7 w-7 cursor-pointer p-1.5 rounded-lg hover:bg-white/10 transform transition-all duration-200 hover:scale-110 active:scale-95" />
    <WindowsArea />
    <div class="flex items-center gap-2">
      <TrayPanel />
      <ClockComponent />
      <img :src="notifyIcon"  alt="Menu" @click="openNotificationCenter"
      class="h-7 w-7 cursor-pointer p-1.5 rounded-lg hover:bg-white/10 transform transition-all duration-200 hover:scale-110 active:scale-95" />
    </div>
  </nav>
</template>

<style scoped>
.panel-nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 2px;
  margin: 0 2px;
  color: white;
  height: 30px;
  border-radius: 12px;
  backdrop-filter: blur(10px);
}

.right-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

.menu-icon {
  height: 28px;
  width: 28px;
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 6px;
  border-radius: 8px;
}

.menu-icon:hover {
  background-color: rgba(255, 255, 255, 0.1);
  transform: scale(1.05);
}
</style>