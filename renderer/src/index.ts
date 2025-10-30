import React from "react";
import type * as RaycastApiType from "@raycast/api";

const LaunchType = {
  UserInitiated: "userInitiated",
  Background: "background",
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
};

export { React, raycastApi };
