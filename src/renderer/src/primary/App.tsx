function App(): JSX.Element {
  const ipcHandle = () => {
    window.electron.ipcRenderer.send("ping");
  };

  return (
    <div className="dark:bg-black h-dvh dark:text-white">
      <button onClick={ipcHandle} hidden></button>

      <div className="grid grid-rows-12 border-2 border-red-500 h-full">
        <header>header</header>
        <main className="row-span-11">
          <div className="grid grid-rows-12 h-full">
            <div className="row-span-8 border-2 border-white">
              <div className="grid grid-rows-1 grid-cols-3 content-center">
                <div className="border-2 border-green-500">
                  <div>schedule</div>
                </div>
                <div className="border-2 border-green-500">
                  <div>preview</div>
                </div>
                <div className="border-2 border-green-500">
                  <div>live</div>
                </div>
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

export default App;
