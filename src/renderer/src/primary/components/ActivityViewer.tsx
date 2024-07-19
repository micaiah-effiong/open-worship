import { assert } from "tsafe";
import React, { HTMLAttributes, PropsWithChildren } from "react";

type ActivityChildrenItemProps<Val> = {
  item: Val;
  isFocused: boolean;
};
// type ActivityChildrenActionProps = {
//   onClick: () => void;
//   onDoubleClick: () => void;
// };
type ActivityChildren<Val> = (
  item: ActivityChildrenItemProps<Val>,
) => React.JSX.Element;

type ActivityViewerProps<T> = {
  children?: ActivityChildren<T>;
  className?: string;
  _viewName?: string;
  onChange?: (event: "select" | "change", item: Nullable<T>) => void;
  itemList: Array<T>;
  defaultItemKey?: Nullable<React.Key>;
};
export type ItemKeyMap<T> = {
  value: T;
  key: React.Key;
};
type Nullable<T> = T | null;
// type HashItemList<T> = { key: React.Key; value: T } & {
//   actions: Record<keyof ActivityChildrenActionProps, () => void>;
// };

export const ActivityViewer = fixedForwardRef(function ActivityViewer<T>(
  props: ActivityViewerProps<{ key: React.Key; value: T }>,
  ref: React.ForwardedRef<HTMLDivElement>,
) {
  const localRef = React.useRef<HTMLDivElement>(null);
  React.useImperativeHandle(ref, () => localRef.current!, []);

  const defaultItemIndex = props.defaultItemKey
    ? props.itemList.findIndex((e) => e.key === props.defaultItemKey)
    : 0;

  assert(
    typeof props.children === "function" ||
      typeof props.children === "undefined",
  );
  const children = props.children || null;

  const onClick = (key: number) => {
    const item = props.itemList[key];
    // console.log("on-click", item);
    if (item) {
      props.onChange?.("select", item);
    }
  };
  const onDoubleClick = (key: number) => {
    const item = props.itemList[key];
    // console.log("on-double-cick", item);
    if (item) {
      props.onChange?.("change", item);
    }
  };

  const roveHandler = (key: string, index: number) => {
    if (key === "Enter") {
      props.onChange?.("change", props.itemList[index]);
    } else {
      onClick(index);
    }
  };

  const [currentFocus, setCurrentFocus] = useRoveFocus(
    localRef,
    props.itemList.length,
    defaultItemIndex,
    roveHandler,
  );

  // React.useEffect(() => {
  //   setSeletectedItem(null);
  // }, [props.itemList, props.defaultItemKey]);

  return (
    <div
      className={`border-2 border-green-500 [&_[data-focus=true]]:focus-within:bg-blue-500 [&_[data-focus=true]]:bg-gray-500 ${props.className}`}
    >
      <div
        className="h-full overflow-y-auto"
        tabIndex={-1}
        ref={localRef}
        role="list"
      >
        {props.itemList.map((item, index) => {
          const isFocused = currentFocus == index;
          return (
            <ListItem
              key={item.key}
              focus={isFocused}
              index={index}
              setFocus={(num: number) => setCurrentFocus(num)}
              onClick={() => onClick(index)}
              onDoubleClick={() => onDoubleClick(index)}
            >
              {currentFocus}
              {children?.({ item, isFocused })}
            </ListItem>
          );
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

type ListItemProps = PropsWithChildren<
  HTMLAttributes<HTMLDivElement> & {
    focus: boolean;
    index: number;
    setFocus: (index: number) => void;
  }
>;

function ListItem(props: ListItemProps) {
  const { setFocus, focus, index, onClick, ...restProps } = props;
  const ref = React.useRef<HTMLDivElement | null>(null);

  const handleSelect = React.useCallback(() => {
    // setting focus to that element when it is selected
    props.setFocus(props.index);
  }, [props.index, props.setFocus]);

  /* React.useEffect(() => {
    if (props.focus && ref.current) {
      ref.current.focus();
    }
  }, [props.focus]); */

  return (
    <div
      role="listitem"
      tabIndex={props.focus ? 0 : -1}
      ref={ref}
      onClick={(e) => {
        onClick?.(e);
        handleSelect();
      }}
      {...restProps}
    >
      {props.children}
    </div>
  );
}

function useRoveFocus<T extends HTMLElement>(
  ref: React.RefObject<T>,
  size: number,
  defaultFocus: number = 0,
  cb?: (key: string, index: number) => void,
) {
  const [currentFocus, setCurrentFocus] = React.useState(0);
  const handleKeyDown = React.useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        const last = size - 1;
        const nextFocus = currentFocus === last ? last : currentFocus + 1;
        setCurrentFocus(nextFocus);
        if (nextFocus !== currentFocus) {
          cb?.(e.key, nextFocus);
        }
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const prevFocus = currentFocus === 0 ? 0 : currentFocus - 1;
        setCurrentFocus(prevFocus);
        if (prevFocus !== currentFocus) {
          cb?.(e.key, prevFocus);
        }
      } else if (e.key === "Enter") {
        cb?.(e.key, currentFocus);
      }
    },
    [size, currentFocus, setCurrentFocus, cb],
  );

  React.useEffect(() => {
    ref.current?.addEventListener("keydown", handleKeyDown, false);
    return () => {
      ref.current?.removeEventListener("keydown", handleKeyDown, false);
    };
  }, [handleKeyDown]);

  React.useEffect(() => {
    setCurrentFocus(defaultFocus);
  }, [defaultFocus]);

  return React.useMemo(() => {
    return [currentFocus, setCurrentFocus] as const;
  }, [currentFocus, setCurrentFocus]);
}
