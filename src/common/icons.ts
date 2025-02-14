import { invoke } from "@tauri-apps/api/core";

export const getIcon = async (name: string): Promise<string> => {
  try {
    return await invoke("get_icon_base64", { name });
  } catch (error) {
    console.error("[Icon Error] Error obteniendo icono:", error);
    return "";
  }
};
