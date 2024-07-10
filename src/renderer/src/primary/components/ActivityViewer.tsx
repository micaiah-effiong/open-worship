import { assert } from "tsafe";
import React, { useState } from "react";

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
export type ItemKeyMap<T> = {
  value: T;
  key: React.Key;
};
type Nullable<T> = T | null;
type HashItemList<T> = { key: React.Key; value: T } & {
  actions: Record<keyof ActivityChildrenActionProps, () => void>;
};

export const ActivityViewer = fixedForwardRef(function ActivityViewer<T>(
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

  /* useEffect(() => {
    let selectedItemValue: Nullable<ItemKeyMap<T>> = null;
    if (selectedItem !== null) {
      selectedItemValue = selectedItem;
    }
    // props.onChange?.(selectedItemValue, "select");
  }, []); */

  return (
    <div
      className={`border-2 border-green-500 [&_[data-focus=true]]:focus-within:bg-blue-500 [&_[data-focus=true]]:bg-gray-500 ${props.className}`}
    >
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
