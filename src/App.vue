<script setup lang="ts">
import SystemTrayComponent from './components/SystemTrayComponent.vue';
import WindowsArea from './components/WindowsArea.vue';
// import { ref } from "vue";
// import { invoke } from "@tauri-apps/api/core";

// const greetMsg = ref("");
// const name = ref("");

// async function greet() {
//   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//   greetMsg.value = await invoke("greet", { name: name.value });
// }
</script>

<template>
  <nav id="navale"> 
    <img src="/usr/share/icons/Fluent/scalable/apps/menu-editor.svg" alt="Menu" class="h-6 w-6"/>
    <WindowsArea />
    <SystemTrayComponent />
  </nav>
</template>

<style>
#navale {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 6px;
  background-color: rgba(0, 0, 0, 0.5);
  color: white;
}
</style>