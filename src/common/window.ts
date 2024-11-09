
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
const thisWindow = await getCurrentWindow();


const setPanelProperties = async () => {
  try {
    const { width: screenWidth } = await window.screen;
    await thisWindow.setSize(new LogicalSize(screenWidth, 32));
  } catch (error) {
    console.error("Error configurando la ventana como panel:", error);
  }
};

export { setPanelProperties };

export default thisWindow;
