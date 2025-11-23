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

// temprary values for testing the pokedex extension
const getPreferenceValues = () => ({
  language: "9",
  duration: "0",
  artwork: "official",
});

export { LaunchType, environment, getPreferenceValues };
