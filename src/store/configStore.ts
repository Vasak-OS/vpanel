import { defineStore } from "pinia";
import { readConfig, type VSKConfig } from "@vasakgroup/plugin-config-manager";
import { ref } from "vue";

export const useConfigStore = defineStore("config", () => {
  const config = ref<VSKConfig | null>(null);

  const loadConfig = async () => {
    config.value = await readConfig();
    if (config.value === null) {
      console.error("Failed to load configuration");
      return;
    }
    setMode();
    setProperties();
  };

  const setMode = () => {
    if (config.value?.style.darkmode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  const setProperties = () => {
    if (config.value) {
      document.documentElement.style.setProperty("--primary-color", config.value.style.color || "#4a90e2");
      document.documentElement.style.setProperty("--border-radius", `${config.value.style.radius}px` || "8px");
    }
  }

  return {
    config,
    loadConfig,
    setMode,
    setProperties
  };
});
