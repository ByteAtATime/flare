import React from "react";
import ReactJsxRuntime, { jsx } from "react/jsx-runtime";
import type * as RaycastApiType from "@raycast/api";
import { invokeCallback, updateContainer } from "./reconciler";
import { Icon } from "./icons";
import { NavigationRoot, useNavigation } from "./navigation";

const LaunchType = {
  UserInitiated: "userInitiated",
  Background: "background",
} as const;

const ToastStyle = {
  Success: "SUCCESS",
  Failure: "FAILURE",
  Animated: "ANIMATED",
} as const;

export type ToastOptions = {
  style?: (typeof ToastStyle)[keyof typeof ToastStyle];
  title: string;
  message?: string;
};

const createComponent = (name: string) => {
  const ComponentFactory = ({
    key,
    ...props
  }: {
    children?: React.ReactNode;
    key?: string | number;
  }) => {
    return jsx(name as React.ElementType, props, key);
  };
  ComponentFactory.displayName = name;
  return ComponentFactory;
};

const Grid = createComponent("Grid");
const GridSection = createComponent("Grid.Section");
const GridItem = createComponent("Grid.Item");

Object.assign(Grid, {
  Section: GridSection,
  Item: GridItem,
});

const ActionPanel = createComponent("ActionPanel");
const ActionPanelSection = createComponent("ActionPanel.Section");

Object.assign(ActionPanel, {
  Section: ActionPanelSection,
});

const Action = createComponent("Action");

const Detail = createComponent("Detail");

class Cache {
  private namespace: string;
  private subscribers: Set<RaycastApiType.Cache.Subscriber>;

  constructor(options?: RaycastApiType.Cache.Options) {
    this.namespace = options?.namespace || "default";
    this.subscribers = new Set();
  }

  public get(key: string): string | undefined {
    const result = rustyscript.functions.cacheGet(this.namespace, key);
    return result === null ? undefined : result;
  }

  public has(key: string): boolean {
    return rustyscript.functions.cacheHas(this.namespace, key);
  }

  public set(key: string, data: string): void {
    rustyscript.functions.cacheSet(this.namespace, key, data);
    this.notifySubscribers(key, data);
  }

  public remove(key: string): boolean {
    const removed = rustyscript.functions.cacheRemove(this.namespace, key);
    if (removed) {
      this.notifySubscribers(key, undefined);
    }
    return removed;
  }

  public clear(
    options: { notifySubscribers: boolean } = { notifySubscribers: true }
  ): void {
    rustyscript.functions.cacheClear(this.namespace);
    if (options.notifySubscribers) {
      this.notifySubscribers(undefined, undefined);
    }
  }

  public get isEmpty(): boolean {
    return rustyscript.functions.cacheIsEmpty(this.namespace);
  }

  public subscribe(
    subscriber: RaycastApiType.Cache.Subscriber
  ): RaycastApiType.Cache.Subscription {
    this.subscribers.add(subscriber);
    return () => {
      this.subscribers.delete(subscriber);
    };
  }

  private notifySubscribers(key: string | undefined, data: string | undefined) {
    for (const subscriber of this.subscribers) {
      try {
        subscriber(key, data);
      } catch (e) {
        console.error("Cache subscriber failed", e);
      }
    }
  }
}

const raycastApi = {
  showToast: (message: string) => {
    console.log(`ooh toast: ${message}`);
  },
  Grid,
  ActionPanel,
  Action,
  Detail,
  LaunchType,
  useNavigation,
  environment: {
    launchType:
      LaunchType.UserInitiated as RaycastApiType.LaunchType.UserInitiated,
    assetsPath: "./test/assets",
  } satisfies Partial<RaycastApiType.Environment>,
  Toast: class {
    public static Style = ToastStyle;

    public primaryAction: RaycastApiType.Toast.ActionOptions | undefined;

    constructor(private options: ToastOptions) {
      console.log(`new toast ${JSON.stringify(options)}`);
    }

    public show = async () => {
      await rustyscript.async_functions.showToast(this.options);
      console.log(`show toast ${JSON.stringify(this.options)}`);
    };
  },
  Cache,
  // temporary defaults for pokédex extension
  getPreferenceValues: () => ({
    language: "9",
    duration: "0",
    artwork: "official",
  }),
  Icon,
};

export {
  React,
  ReactJsxRuntime,
  NavigationRoot,
  raycastApi,
  updateContainer,
  invokeCallback,
};
