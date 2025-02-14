<script setup lang="ts">
import { onMounted } from "vue";
import { setPanelProperties } from "@/common/window";
import SystemTrayComponent from '@/components/SystemTrayComponent.vue';
import WindowsArea from '@/components/WindowsArea.vue';

onMounted(async () => {
  await setPanelProperties();
});
</script>

<template>
  <nav class="panel-nav"> 
    <img 
      src="/usr/share/icons/Fluent/scalable/apps/menu-editor.svg" 
      alt="Menu" 
      class="menu-icon"
    />
    <WindowsArea />
    <SystemTrayComponent />
  </nav>
</template>

<style scoped>
.panel-nav {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 2px;
  margin: 0 2px ;
  background-color: rgba(0, 0, 0, 0.5);
  color: white;
  height: 30px;
  border-radius: 12px;
  backdrop-filter: blur(10px);
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