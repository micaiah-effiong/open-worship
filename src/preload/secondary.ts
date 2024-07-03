import { electronAPI } from "@electron-toolkit/preload";
import { contextBridge } from "electron";

// Custom APIs for renderer
const api = {
  ping: () => {
    console.log("trying to invoke ping");
    electronAPI.ipcRenderer.send("ping");
  },
};

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld("api", api);
  } catch (error) {
    console.error(error);
  }
} else {
  // @ts-ignore (define in dts)
  // @ts-ignore (define in dts)
  window.api = api;
}
