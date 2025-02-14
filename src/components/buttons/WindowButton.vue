<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getIcon } from "@/common/icons";

interface Props {
  id: string;
  title: string;
  is_minimized: boolean;
  icon: string;
}

const props = defineProps<Props>();
const iconBase64 = ref<string>("");

const toggleWindow = async (): Promise<void> => {
  try {
    await invoke("toggle_window", { windowId: props.id });
  } catch (error) {
    console.error("[Window Error] Error alternando ventana:", error);
  }
};

onMounted(async () => {
  if (props.icon) {
    iconBase64.value = await getIcon(props.icon);
  }
});
</script>

<template>
  <div
    class="window-button"
    :class="{ 'window-minimized': is_minimized }"
    @click="toggleWindow"
  >
    <img 
      v-if="icon && iconBase64" 
      :src="`data:image/svg+xml;base64,${iconBase64}`" 
      :alt="title"
      class="window-icon"
    />
    <div v-else class="icon-placeholder" />
  </div>
</template>

<style scoped>
.window-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 8px;
}

.window-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
  transform: scale(1.05);
}

.window-minimized {
  opacity: 0.5;
}

.window-icon {
  width: 24px;
  height: 24px;
}

.icon-placeholder {
  width: 24px;
  height: 24px;
  background-color: rgba(128, 128, 128, 0.5);
  border-radius: 6px;
}
</style> 