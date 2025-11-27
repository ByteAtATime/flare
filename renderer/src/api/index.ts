import { Grid, List, ActionPanel, Action, Detail } from "./components";
import { Toast } from "./toast";
import { Cache } from "./cache";
import { LaunchType, environment, getPreferenceValues } from "./environment";
import { useNavigation } from "./navigation";
import { Icon } from "./icons";
import * as protocol from "../protocol";

const openExtensionPreferences = () => protocol.openExtensionPreferences();
const openCommandPreferences = () => protocol.openCommandPreferences();

const raycastApi = {
  showToast: (message: string) => {},
  Grid,
  List,
  ActionPanel,
  Action,
  Detail,
  LaunchType,
  useNavigation,
  environment,
  Toast,
  Cache,
  getPreferenceValues,
  Icon,
  openExtensionPreferences,
  openCommandPreferences,
  Color: {
    Blue: "raycast-blue",
    Green: "raycast-green",
    Magenta: "raycast-magenta",
    Orange: "raycast-orange",
    Purple: "raycast-purple",
    Red: "raycast-red",
    Yellow: "raycast-yellow",

    PrimaryText: "raycast-primary-text",
    SecondaryText: "raycast-secondary-text",
  },
};

export { raycastApi };
