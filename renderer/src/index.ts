import React from "react";
import type * as RaycastApiType from "@raycast/api";

const LaunchType = {
  UserInitiated: "userInitiated",
  Background: "background",
} as const;

const ToastStyle = {
  Success: "SUCCESS",
  Failure: "FAILURE",
  Animated: "ANIMATED",
} as const;

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

    constructor(
      private options: {
        style?: (typeof ToastStyle)[keyof typeof ToastStyle];
        title: string;
        message?: string;
      }
    ) {
      console.log(`new toast ${JSON.stringify(options)}`);
    }

    public show = () => {
      console.log(`show toast ${JSON.stringify(this.options)}`);
    };
  },
};

export { React, raycastApi };
