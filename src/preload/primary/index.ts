import { contextBridge } from "electron";
import { electronAPI, IpcRendererListener } from "@electron-toolkit/preload";

const chan = ["go_live"] as const;
const channels = chan.map((e) => "primary::" + e);
type i_channels = `primary::${(typeof chan)[number]}`;

// Custom APIs for renderer
const api = {
  go_live: () => {
    electronAPI.ipcRenderer.send("primary:go_live");
  },

  listen: (channel: i_channels, func: IpcRendererListener) => {
    if (channels.includes(channel)) {
      const sub: IpcRendererListener = (event, ...args) => {
        return func(event, ...args);
      };
      return electronAPI.ipcRenderer.on(channel, sub);
    }

    return () => {};
  },
};

export type Api = typeof api;

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld("electron", electronAPI);
    contextBridge.exposeInMainWorld("api", api);
  } catch (error) {
    console.error(error);
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI;
  // @ts-ignore (define in dts)
  window.api = api;
}
