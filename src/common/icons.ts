import { invoke } from "@tauri-apps/api/core";

export const getIcon = async (name: string): Promise<string> => {
  try {
    return await invoke("get_icon_base64", { name });
  } catch (error) {
    console.error("[Icon Error] Error obteniendo icono:", error);
    return "";
  }
};

export const getImageType = (base64String: string): string => {
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