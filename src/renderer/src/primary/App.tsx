import { useRef, useState } from "react";
import { ActivityViewer, ItemKeyMap } from "./components/ActivityViewer";
import { ScreenViewer } from "@renderer/general/components/ScreenViewer";

type DisplayItem = {
  text: string;
};
type ItemMapDisplay = ItemKeyMap<DisplayItem>;

type LiveRenderList = [
  Array<ItemKeyMap<DisplayItem>>,
  ItemKeyMap<DisplayItem> | null,
];

const defaultLive: Array<ItemKeyMap<DisplayItem>> = [];
const previewData = [
  {
    key: "1",
    value: { text: "the song" },
  },
  {
    key: "2",
    value: {
      text: "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.",
    },
  },
];
const scheduleData = [
  {
    key: "1",
    value: [
      {
        key: "1.1",
        value: { text: "schedule 1 a" },
      },
      {
        key: "1.2",
        value: { text: "schedule 1 b" },
      },
      {
        key: "1.3",
        value: { text: "schedule 1 c" },
      },
    ],
  },
  {
    key: "2",
    value: [
      {
        key: "2.1",
        value: { text: "schedule 2 a" },
      },
      {
        key: "2.2",
        value: { text: "schedule 2 b" },
      },
      {
        key: "2.3",
        value: { text: "schedule 2 c" },
      },
      {
        key: "2.4",
        value: { text: "schedule 2 d" },
      },
    ],
  },
];

export function App(): JSX.Element {
  const [scheduleList, _setScheduleList] =
    useState<ItemKeyMap<ItemMapDisplay[]>[]>(scheduleData);
  const [previewList, setPreviewList] = useState<ItemMapDisplay[]>(previewData);
  const [liveList, setLiveList] = useState<LiveRenderList>([defaultLive, null]);
  const liveViewerRef = useRef<HTMLDivElement>(null);

  const [previewText, setPreviewText] = useState(previewData[0].value.text);
  const [liveText, setLiveText] = useState("");

  const ipcHandle = () => {
    window.electron.ipcRenderer.send("ping");
  };

  const goLive = (msg: string) => {
    setLiveText(msg);
    window.electron.ipcRenderer.send("primary::go_live", msg);
  };

  return (
    <div className="h-dvh bg-gray-500/50">
      <button onClick={ipcHandle} hidden></button>

      <div className="grid grid-rows-12 border-2 border-red-500 h-full">
        <header>header</header>
        <main className="row-span-11">
          <div className="grid grid-rows-12 h-full">
            <div className="row-span-8 border-2 border-black">
              <div className="grid grid-rows-1 grid-cols-3 h-full content-center">
                <ActivityViewer
                  _viewName="schedule"
                  onChange={(event, item) => {
                    if (!item?.value) {
                      return;
                    }

                    if (event === "select") {
                      setPreviewList(item.value);
                    } else if (event === "change") {
                      setLiveList([item.value, item.value[0]]);
                      goLive(item.value[0].value.text || "");
                      liveViewerRef.current?.focus();
                      setPreviewText(item?.value[0].value.text || "");
                    }
                  }}
                  itemList={scheduleList}
                >
                  {(value) => {
                    return (
                      <div key={value.item.key} className="grid">
                        <button
                          className="text-left whitespace-pre-line"
                          data-focus={value.isFocused}
                        >
                          {value?.item?.value[0].value.text}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
                <ActivityViewer
                  _viewName="preview"
                  onChange={(event, item) => {
                    if (event === "select") {
                      // setLiveList(previewList);
                      setPreviewText(item?.value.text || "");
                    } else if (event === "change") {
                      setLiveList([previewList, previewList[1]]);
                      setTimeout(() => {
                        setLiveList([previewList, item]);
                      }, 0);
                      goLive(item?.value.text || "");
                      liveViewerRef.current?.focus();
                    }
                  }}
                  defaultItemKey={previewList[0].key}
                  itemList={previewList}
                >
                  {(value) => {
                    return (
                      <div key={value.item.key.toString()} className="grid">
                        <button
                          className="text-left whitespace-pre-line"
                          data-focus={value.isFocused}
                        >
                          {value.item.value.text} {String(value.isFocused)}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
                <ActivityViewer
                  _viewName="live"
                  ref={liveViewerRef}
                  onChange={(event, item) => {
                    if (event === "change") {
                      return;
                    }

                    goLive(item?.value?.text || "");
                  }}
                  itemList={liveList[0]}
                  defaultItemKey={liveList[1]?.key}
                >
                  {(value) => {
                    return (
                      <div key={value.item.key} className="grid">
                        <button
                          className="text-left whitespace-pre-line"
                          data-focus={value.isFocused}
                        >
                          {value.item.value.text} {String(value.isFocused)}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
              </div>
            </div>
            <div className="row-span-4 border-2 border-sky-500">
              <div className="grid grid-cols-3 border-8 border-dotted border-pink-500 h-full">
                <div className="border-2 border-green-500 h-full grid grid-rows-12">
                  <div className="row-span-1">song/bible/backgrounds</div>
                  <div className="border border-black row-span-11 flex flex-col">
                    <div className="">
                      <input
                        type="search"
                        placeholder="search/list"
                        className="bg-gray-800 w-full"
                      />
                    </div>
                    <div className="border-dashed border-2 border-rose-900 h-full">
                      search preview
                    </div>
                  </div>
                </div>
                <div className="col-span-2 h-full border-2 border-green-500">
                  <div className="grid grid-cols-2 border-2 h-full border-indigo-900">
                    <div className="border-2 border-zinc-500">
                      <ScreenViewer text={previewText} />
                    </div>
                    <div className="border-2 border-zinc-500">
                      <ScreenViewer text={liveText} />
                    </div>
                  </div>
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

// schedule -> preview
// schedule -> preview
// preview -> live
// liveview -> live
