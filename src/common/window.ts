
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { moveWindow, Position } from "@tauri-apps/plugin-positioner";

const thisWindow = await getCurrentWindow();


const setPanelProperties = async (position: Position) => {
  try {
    const { width: screenWidth } = await window.screen;

    await thisWindow.setSize(new LogicalSize(screenWidth, 32));
    await thisWindow.setMaxSize(new LogicalSize(screenWidth, 32));
    await thisWindow.setMinSize(new LogicalSize(screenWidth, 32));
    await moveWindow(position);
  } catch (error) {
    console.error("Error configurando la ventana como panel:", error);
  }
};

export { setPanelProperties };

export default thisWindow;
