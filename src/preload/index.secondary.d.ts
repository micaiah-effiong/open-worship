import { ElectronAPI } from "@electron-toolkit/preload";

declare global {
  interface Window {
    api: {
      ping: () => void;
    };
  }
}
