import { invoke } from "@tauri-apps/api/core";

const getIcon = async (name: string): Promise<string> => {
  return await invoke("get_icon_base64", { name });
};

export { getIcon };
