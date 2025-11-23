import type * as RaycastApiType from "@raycast/api";
import * as protocol from "../protocol";

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

class Toast {
  public static Style = ToastStyle;

  public primaryAction: RaycastApiType.Toast.ActionOptions | undefined;

  constructor(private options: ToastOptions) {}

  public show = async () => {
    await protocol.showToast(this.options);
  };
}

export { Toast, ToastStyle };
