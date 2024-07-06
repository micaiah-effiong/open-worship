import { PropsWithChildren } from "react";

export function App(): JSX.Element {
  const ipcHandle = () => {
    window.electron.ipcRenderer.send("ping");
  };

  const goLive = (msg: string) => {
    window.electron.ipcRenderer.send("primary::go_live", msg);
  };

  return (
    <div className="h-dvh bg-gray-500">
      <button onClick={ipcHandle} hidden></button>

      <div className="grid grid-rows-12 border-2 border-red-500 h-full">
        <header>header</header>
        <main className="row-span-11">
          <div className="grid grid-rows-12 h-full">
            <div className="row-span-8 border-2 border-black">
              <div className="grid grid-rows-1 grid-cols-3 h-full content-center">
                <Viewer>
                  <div>schedule</div>
                </Viewer>
                <Viewer className="grid">
                  <button onClick={() => goLive("preview")}>preview</button>
                  <button onClick={() => goLive("songs")}>songs</button>
                  <button onClick={() => goLive("Scriptures")}>
                    Scriptures
                  </button>
                  <button onClick={() => goLive("presentation")}>
                    presentation
                  </button>
                  <button onClick={() => goLive("theme")}>theme</button>
                </Viewer>
                <Viewer>
                  <div>live</div>
                </Viewer>
              </div>
            </div>
            <div className="row-span-4 border-2 border-sky-500">
              <div className="grid grid-cols-3">
                <div className="border-2 border-green-500">
                  <div className="">search/list</div>
                </div>
                <div className="col-span-2 border-2 border-green-500">
                  <div className="">search preview</div>
                </div>
              </div>
            </div>
          </div>
        </main>
        <footer> footer </footer>
      </div>
    </div>
  );
}

function Viewer(props: PropsWithChildren & { className?: string }) {
  return (
    <div className={`border-2 border-green-500 ${props.className}`}>
      <div className="h-full overflow-y-auto">{props.children}</div>
    </div>
  );
}
