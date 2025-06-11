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
    class="vpanel">
    <img :src="menuIcon"  alt="Menu" @click="openMenu"
      class="app-icon" />
    <WindowsArea />
    <div class="flex content-center items-center">
      <TrayPanel />
      <ClockComponent />
      <img :src="notifyIcon"  alt="Menu" @click="openNotificationCenter"
      class="app-icon" />
    </div>
  </nav>
</template>

<style>
@reference "./style.css";

.vpanel {
  @apply flex justify-between items-center mx-1 bg-white/70 dark:bg-black/70 hover:bg-white/80 hover:dark:bg-black/80;
  height: 30px;
  padding: 2px;
  border-radius: calc(var(--vsk-border-radius) + 2px);
}

.vpanel .app-icon {
  @apply h-6 w-6 cursor-pointer p-0.5 rounded-vsk hover:bg-vsk-primary/30 transform hover:scale-110 active:scale-95;
}
</style>