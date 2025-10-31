import React from "react";
import ReactJsxRuntime, { jsx } from "react/jsx-runtime";
import type * as RaycastApiType from "@raycast/api";
import { updateContainer } from "./reconciler";

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
  const ComponentFactory = (props: { children?: React.ReactNode }) => {
    return jsx(name as React.ElementType, props);
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

const raycastApi = {
  showToast: (message: string) => {
    console.log(`ooh toast: ${message}`);
  },
  Grid,
  LaunchType,
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
};

export { React, ReactJsxRuntime, raycastApi, updateContainer };
