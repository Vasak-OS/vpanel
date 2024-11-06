<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { getIcon } from "../../common/icons";
import { onMounted, ref } from "vue";

const iconBase64 = ref<string>("");

const { id, title, is_minimized, icon } = defineProps({
  id: {
    type: String,
    required: true,
  },
  title: String,
  is_minimized: Boolean,
  icon: {
    type: String,
    default: "",
  },
});

const focusWindow = async (windowId: string) => {
  try {
    await invoke("focus_window", { windowId });
  } catch (error) {
    console.error("Error focusing window:", error);
  }
};

onMounted(async () => {
  iconBase64.value = await getIcon(icon);
});
</script>
<template>
  <div
    class="px-3 py-1 hover:bg-gray-700 rounded cursor-pointer flex items-center gap-2"
    :class="{ 'opacity-50': is_minimized }"
    @click="focusWindow(id)"
  >
    <img v-if="icon" :src="`data:image/svg+xml;base64,${iconBase64}`" class="w-6 h-6" :alt="title" />
    <span class="hidden">{{ title }}</span>
  </div>
</template>

<style>
</style>