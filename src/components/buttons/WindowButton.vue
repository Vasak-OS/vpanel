<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getIconSource } from "@vasakgroup/plugin-vicons";

interface Props {
  id: string;
  title: string;
  is_minimized: boolean;
  icon: string;
}

const props = defineProps<Props>();
const iconSource = ref<string>("");

const toggleWindow = async (): Promise<void> => {
  try {
    await invoke("toggle_window", { windowId: props.id });
  } catch (error) {
    console.error("[Window Error] Error alternando ventana:", error);
  }
};

onMounted(async () => {
  if (props.icon) {
    iconSource.value = await getIconSource(props.icon);
  }
});
</script>

<template>
  <div
    class="window-button"
    :class="{ 'opacity-50 hover:opacity-90': is_minimized }"
    @click="toggleWindow"
  >
    <img 
      v-if="icon && iconSource" 
      :src="iconSource" 
      :alt="title"
      class="w-6 h-6 transition-all duration-300 group-hover:rotate-3 group-hover:brightness-110"
    />
    <div 
      v-else 
      class="w-6 h-6 bg-gray-500/50 rounded-md animate-pulse" 
    />
    <div class="absolute -top-2 z-20 opacity-0 group-hover:opacity-100 transition-all duration-300 text-xs px-2 py-0.5 bg-black/80 rounded-md whitespace-nowrap pointer-events-none transform group-hover:translate-y-5 backdrop-blur-sm">
      {{ title }}
    </div>
  </div>
</template>

<style>
@reference "../../style.css";

.window-button {
  @apply flex items-center justify-center w-7 h-7 cursor-pointer transform rounded-vsk hover:bg-vsk-primary/30 hover:scale-110 active:scale-95 relative;
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