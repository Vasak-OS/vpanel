<script setup lang="ts">
import { onMounted } from "vue";
import { setPanelProperties } from "@/common/window";
import WindowsArea from '@/components/WindowsArea.vue';
import TrayPanel from '@/components/TrayPanel.vue';
import ClockComponent from '@/components/ClockComponent.vue';

onMounted(async () => {
  await setPanelProperties();
});
</script>

<template>
  <nav class="flex justify-between items-center px-1 mx-1 bg-black/50 text-white h-[30px] rounded-xl backdrop-blur-md transition-all duration-300 hover:bg-black/60"> 
    <img 
      src="/usr/share/icons/Fluent/scalable/apps/menu-editor.svg" 
      alt="Menu" 
      class="h-7 w-7 cursor-pointer p-1.5 rounded-lg hover:bg-white/10 transform transition-all duration-200 hover:scale-110 active:scale-95"
    />
    <WindowsArea />
    <div class="flex items-center gap-2">
      <TrayPanel />
      <ClockComponent />
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
  background-color: rgba(0, 0, 0, 0.5);
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