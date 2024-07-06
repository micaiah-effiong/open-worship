import { electronAPI, IpcRendererListener } from "@electron-toolkit/preload";
import { contextBridge } from "electron";

const chan = ["go_live", "test"] as const;
const channels = chan.map((e) => "secondary::" + e);
type i_channels = `secondary::${(typeof chan)[number]}`;

// Custom APIs for renderer
const api = {
  ping: () => {
    console.log("trying to invoke ping");
    electronAPI.ipcRenderer.send("ping");
  },

  listen: (channel: i_channels, func: IpcRendererListener) => {
    if (channels.includes(channel)) {
      const sub: IpcRendererListener = (event, ...args) => {
        console.log(channel, "event recieved");
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
    contextBridge.exposeInMainWorld("api", api);
  } catch (error) {
    console.error(error);
  }
} else {
  // @ts-ignore (define in dts)
  // @ts-ignore (define in dts)
  window.api = api;
}
