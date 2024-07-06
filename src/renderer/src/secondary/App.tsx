import { useEffect, useState } from "react";

export function App() {
  const [liveText, setLiveText] = useState<string | null>(null);

  useEffect(() => {
    return window.api.listen("secondary::go_live", (evt, data, otherData) => {
      console.log("go::live", evt, data, otherData);
      setLiveText(data);
    });
  }, []);

  return (
    <div className="border-red-500 border-5 h-dvh w-full grid grid-rows-1">
      <div className="grid place-items-center">
        <div className="text-center text-9xl text-white outline-black outline-4">
          {liveText}
        </div>
      </div>
    </div>
  );
}
