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

const getImageType = (base64String: string): string => {
  try {
    // Decodificar los primeros bytes del base64
    const binaryString = atob(base64String.substring(0, 32));
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    // Verificar si es PNG (signature bytes)
    if (
      bytes[0] === 0x89 &&
      bytes[1] === 0x50 &&
      bytes[2] === 0x4E &&
      bytes[3] === 0x47
    ) {
      return 'image/png';
    }

    // Por defecto, asumir SVG
    return 'image/svg+xml';
  } catch (error) {
    console.error('Error identificando tipo de imagen:', error);
    return 'image/png';
  }
};

onMounted(async () => {
  if (props.icon) {
    iconBase64.value = await getIcon(props.icon);
    console.log(props.icon);
    console.log(iconBase64);
  }
});
</script>

<template>
  <div
    class="group flex items-center justify-center w-10 h-10 cursor-pointer transform transition-all duration-300 rounded-lg hover:bg-white/10 hover:scale-110 active:scale-95 relative"
    :class="{ 'opacity-50 hover:opacity-90': is_minimized }"
    @click="toggleWindow"
  >
    <img 
      v-if="icon && iconBase64" 
      :src="`data:${getImageType(iconBase64)};base64,${iconBase64}`" 
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
  transform: scale(1.3);
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