import React from "react";
import ReactJsxRuntime from "react/jsx-runtime";
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

const raycastApi = {
  showToast: (message: string) => {
    console.log(`ooh toast: ${message}`);
  },
  LaunchType,
  environment: {
    launchType:
      LaunchType.UserInitiated as RaycastApiType.LaunchType.UserInitiated,
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
