import { useRef, useState } from "react";
import clsx from "clsx";
import { ActivityViewer, ItemKeyMap } from "./components/ActivityViewer";

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
        key: "1",
        value: { text: "schedule 1 a" },
      },
      {
        key: "2",
        value: { text: "schedule 1 b" },
      },
      {
        key: "3",
        value: { text: "schedule 1 c" },
      },
    ],
  },
  {
    key: "2",
    value: [
      {
        key: "1",
        value: { text: "schedule 2 a" },
      },
      {
        key: "2",
        value: { text: "schedule 2 b" },
      },
      {
        key: "3",
        value: { text: "schedule 2 c" },
      },
    ],
  },
];

export function App(): JSX.Element {
  const [scheduleList, _setScheduleList] =
    useState<ItemKeyMap<ItemMapDisplay[]>[]>(scheduleData);
  const [previewList, _setPreviewList] =
    useState<ItemMapDisplay[]>(previewData);
  const [liveList, setLiveList] = useState<LiveRenderList>([defaultLive, null]);
  const liveViewerRef = useRef<HTMLDivElement>(null);

  const ipcHandle = () => {
    window.electron.ipcRenderer.send("ping");
  };

  const goLive = (msg: string) => {
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
                  onChange={(item, event) => {
                    if (!item?.value) {
                      return;
                    }

                    if (event === "select") {
                      _setPreviewList(item?.value);
                    } else if (event === "change") {
                      setLiveList([item.value, item.value[0]]);
                      goLive(item?.value[0].value.text || "");
                      liveViewerRef.current?.focus();
                    }
                  }}
                  itemList={scheduleList}
                >
                  {(value, action) => {
                    return (
                      <div key={value.item.key} className="grid">
                        <button
                          {...action}
                          className={clsx("text-left whitespace-pre-line")}
                          data-focus={value.isFocused}
                        >
                          {value?.item?.value[0].value.text}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
                <ActivityViewer
                  onChange={(item, event) => {
                    if (event === "select") {
                      // setLiveList(previewList);
                    } else if (event === "change") {
                      setLiveList([previewList, item]);
                      goLive(item?.value.text || "");
                      liveViewerRef.current?.focus();
                    }
                  }}
                  itemList={previewList}
                >
                  {(value, action) => {
                    return (
                      <div key={value.item.key} className="grid">
                        <button
                          {...action}
                          className={clsx("text-left whitespace-pre-line")}
                          data-focus={value.isFocused}
                        >
                          {value?.item?.value?.text}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
                <ActivityViewer
                  ref={liveViewerRef}
                  onChange={(item) => goLive(item?.value?.text || "")}
                  itemList={liveList[0]}
                  defaultItemKey={liveList[1]?.key}
                >
                  {(value, action) => {
                    return (
                      <div key={value.item.key} className="grid">
                        <button
                          {...action}
                          className={clsx("text-left whitespace-pre-line")}
                          data-focus={value.isFocused}
                        >
                          {value.item.value.text}
                        </button>
                      </div>
                    );
                  }}
                </ActivityViewer>
              </div>
            </div>
            <div className="row-span-4 border-2 border-sky-500">
              <div className="grid grid-cols-3 border-2 border-pink-500 h-full">
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

// schedule -> preview
// schedule -> preview
// preview -> live
// liveview -> live
