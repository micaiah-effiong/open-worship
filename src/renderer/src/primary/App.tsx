import { assert } from "tsafe";
import React, { PropsWithChildren, useEffect, useRef, useState } from "react";
import clsx from "clsx";

type DisplayItem = {
  text: string;
};
type LiveRenderList = [
  Array<ItemKeyMap<DisplayItem>>,
  ItemKeyMap<DisplayItem> | null,
];

const defaultLive: Array<ItemKeyMap<DisplayItem>> = [];
const data = [
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

export function App(): JSX.Element {
  const [previewList, _setPreviewList] =
    useState<Array<ItemKeyMap<DisplayItem>>>(data);
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
                <Viewer>
                  <div>schedule</div>
                </Viewer>
                <ActivityViewer
                  className="[&_[data-focus=true]]:focus-within:bg-blue-500 [&_[data-focus=true]]:bg-gray-500"
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
                  // className="focus-within:text-green-500"
                  className="[&_[data-focus=true]]:focus-within:bg-blue-500 [&_[data-focus=true]]:bg-gray-500"
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

function Viewer(props: PropsWithChildren & { className?: string }) {
  return (
    <div className={`border-2 border-green-500 ${props.className}`}>
      <div className="h-full overflow-y-auto">{props.children}</div>
    </div>
  );
}

type ActivityChildrenItemProps<Val> = {
  item: Val;
  isFocused: boolean;
};
type ActivityChildrenActionProps = {
  onClick: () => void;
  // onSelect: (key: number) => void;
  onDoubleClick: () => void;
};
type ActivityChildren<Val> = (
  item: ActivityChildrenItemProps<Val>,
  action: ActivityChildrenActionProps,
) => React.JSX.Element;

type ActivityViewerProps<T> = {
  children?: ActivityChildren<T>;
  className?: string;
  onChange?: (item: Nullable<T>, event: "select" | "change") => void;
  itemList: Array<T>;
  defaultItemKey?: Nullable<React.Key>;
};
type ItemKeyMap<T> = {
  value: T;
  key: React.Key;
};
type Nullable<T> = T | null;
type HashItemList<T> = { key: React.Key; value: T } & {
  actions: Record<keyof ActivityChildrenActionProps, () => void>;
};

const ActivityViewer = fixedForwardRef(function ActivityViewer<T>(
  props: ActivityViewerProps<{ key: React.Key; value: T }>,
  ref: React.ForwardedRef<HTMLDivElement>,
) {
  const defaultItem = props.defaultItemKey
    ? props.itemList.find((e) => e.key === props.defaultItemKey)
    : props.itemList.at(0);
  const [selectedItem, setSeletectedItem] =
    useState<Nullable<ItemKeyMap<T>>>(null);

  // console.log("default", defaultItem, "kwy", props.defaultItemKey);

  assert(typeof props.children === "function");
  const children = props.children; // as ActivityChildren<T>;

  const itemMap = new Map<React.Key, ItemKeyMap<T>>();
  const hashedItems = props.itemList.map((item) => {
    itemMap.set(item.key, item);

    return {
      ...item,
      actions: {
        onClick: () => {
          onClick(item.key);
        },
        onDoubleClick: () => {
          onDoubleClick(item.key);
        },
      },
    } as HashItemList<T>;
  });

  const onClick = (key: React.Key) => {
    const item = itemMap.get(key);
    // console.log("on-click", item);
    if (item) {
      setSeletectedItem(item);
      props.onChange?.(item, "select");
    }
  };
  const onDoubleClick = (key: React.Key) => {
    const item = itemMap.get(key);
    // console.log("on-double-cick", item);
    if (item) {
      setSeletectedItem(item);
      props.onChange?.(item, "change");
    }
  };

  useEffect(() => {
    let selectedItemValue: Nullable<ItemKeyMap<T>> = null;
    if (selectedItem !== null) {
      selectedItemValue = selectedItem;
    }
    props.onChange?.(selectedItemValue, "select");
  }, []);

  return (
    <div className={`border-2 border-green-500 ${props.className}`}>
      <div className="h-full overflow-y-auto" tabIndex={-1} ref={ref}>
        {hashedItems.map((item) => {
          const key = selectedItem?.key || defaultItem?.key;
          return children({ item, isFocused: item.key === key }, item.actions);
        })}
      </div>
    </div>
  );
});

function fixedForwardRef<T, P = {}>(
  render: (props: P, ref: React.Ref<T>) => React.ReactNode,
): (props: P & React.RefAttributes<T>) => React.ReactNode {
  return React.forwardRef(render) as any;
}
