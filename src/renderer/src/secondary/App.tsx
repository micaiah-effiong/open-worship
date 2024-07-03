export function App() {
  const ipcHandle = () => {
    window.api.ping();
  };

  return (
    <div>
      <a target="_blank" rel="noreferrer" onClick={ipcHandle}>
        Send IPC
      </a>
    </div>
  );
}
