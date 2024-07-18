import electron, { app, shell, BrowserWindow, ipcMain } from "electron";
import { join } from "path";
import { electronApp, optimizer, is } from "@electron-toolkit/utils";
import icon from "../../resources/icon.png?asset";
import { db } from "./db";

db;

function createMainWindow() {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    // width: 900,
    // height: 670,
    // show: false,
    autoHideMenuBar: true,
    ...(process.platform === "linux" ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, "../preload/primary.js"),
      sandbox: false,
    },
  });

  mainWindow.on("ready-to-show", () => {
    // mainWindow.show();
    mainWindow.maximize();
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  // HMR for renderer base on electron-vite cli.
  // Load the remote URL for development or the local html file for production.
  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(process.env["ELECTRON_RENDERER_URL"]);
  } else {
    mainWindow.loadFile(join(__dirname, "../renderer/index.html"));
  }

  return mainWindow;
}

console.log(process.env["ELECTRON_RENDERER_URL"], { "is.dev": is.dev });

function createSecondaryWindow() {
  const screens = electron.screen.getAllDisplays();
  //
  // Create the browser window.
  const secondaryWindow = new BrowserWindow({
    frame: false,
    show: false,
    autoHideMenuBar: true,
    ...(process.platform === "linux" ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, "../preload/secondary.js"),
      sandbox: false,
    },
  });

  secondaryWindow.on("ready-to-show", () => {
    secondaryWindow.maximize();
    const screenIndex = screens.length > 1 ? 1 : 0;
    const secScreen = screens[screenIndex];

    if (screenIndex > 1) {
      secondaryWindow.setSize(secScreen.size.width, secScreen.size.height);
      secondaryWindow.setPosition(secScreen.bounds.x, secScreen.bounds.y);
      secondaryWindow.setFullScreen(true);
    } else {
      const width = screens[0].bounds.width / 2;
      const height = screens[0].bounds.height / 2;
      secondaryWindow.setSize(width, height, false);
      secondaryWindow.setPosition(0, 0, false);
      secondaryWindow.setFullScreen(false);
    }

    secondaryWindow.show();
  });

  secondaryWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  // HMR for renderer base on electron-vite cli.
  // Load the remote URL for development or the local html file for production.
  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    secondaryWindow.loadURL(
      process.env["ELECTRON_RENDERER_URL"] + "/secondary.html",
    );
  } else {
    secondaryWindow.loadFile(join(__dirname, "../renderer/secondary.html"));
  }

  return secondaryWindow;
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  // Set app user model id for windows
  electronApp.setAppUserModelId("com.electron");
  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  // console.log(electron.screen.getAllDisplays());

  // IPC test
  ipcMain.on("ping", () => console.log("pong"));

  const secondaryScreen = createSecondaryWindow();
  createMainWindow();

  ipcMain.on("primary::go_live", (_event, ...args) => {
    secondaryScreen.webContents.send("secondary::go_live", ...args);
  });

  app.on("activate", function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) {
      createSecondaryWindow();
      createMainWindow();
    }
  });
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

// In this file you can include the rest of your app"s specific main process
// code. You can also put them in separate files and require them here.
