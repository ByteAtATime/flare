import type * as RaycastApiType from "@raycast/api";

const LaunchType = {
  UserInitiated: "userInitiated",
  Background: "background",
} as const;

const environment = {
  launchType:
    LaunchType.UserInitiated as RaycastApiType.LaunchType.UserInitiated,
  assetsPath: "./test/assets",
} satisfies Partial<RaycastApiType.Environment>;

let preferenceValues: Record<string, unknown> = {};

export const setPreferences = (prefs: Record<string, unknown>) => {
  preferenceValues = prefs;
};

const getPreferenceValues = <T extends Record<string, unknown>>(): T => {
  return preferenceValues as T;
};

export { LaunchType, environment, getPreferenceValues };
