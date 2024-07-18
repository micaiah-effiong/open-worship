import { resolve } from "path";
import { defineConfig, externalizeDepsPlugin } from "electron-vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      lib: {
        entry: {
          primary: resolve("src/preload/primary/index.ts"),
          secondary: resolve("src/preload/secondary/index.ts"),
        },
      },
    },
  },
  renderer: {
    resolve: {
      alias: {
        "@renderer": resolve("src/renderer/src"),
        // "@primary": resolve("src/renderer/src/primary/components"),
        // "@secondary": resolve("src/renderer/src/secondary/components"),
      },
    },
    build: {
      rollupOptions: {
        input: {
          primary: resolve(__dirname, "src/renderer/index.html"),
          secondary: resolve(__dirname, "src/renderer/secondary.html"),
        },
      },
    },
    plugins: [react()],
  },
});
