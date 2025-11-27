import { useCallback, type FunctionComponent, type ReactNode } from "react";
import { useNavigation } from "./navigation";
import { Action } from "./components";
import * as protocol from "../protocol";
import { Icon } from "./icons";

type PushProps = {
  title: string;
  target: ReactNode;
  icon?: unknown; // TODO
  shortcut?: unknown; // TODO
  onPush?: () => void;
  onPop?: () => void;
};

export const Push: FunctionComponent<PushProps> = (props) => {
  const { title, icon, shortcut, onPush, onPop, target } = props;
  const { push } = useNavigation();

  const handleAction = useCallback(() => {
    push(target, onPop);
    onPush?.();
  }, [target, onPush, push, onPop]);

  return (
    <Action
      title={title}
      icon={icon}
      shortcut={shortcut}
      onAction={handleAction}
    />
  );
};

Push.displayName = "Action.Push";

type CopyToClipboardProps = {
  content: string | number;
  title?: string;
  icon?: unknown;
  shortcut?: unknown;
  onCopy?: (content: string | number) => void;
};

export const CopyToClipboard: FunctionComponent<CopyToClipboardProps> = (
  props
) => {
  const {
    content,
    title = "Copy to Clipboard",
    icon = Icon.Clipboard,
    shortcut,
    onCopy,
  } = props;

  const handleAction = useCallback(async () => {
    await protocol.copyToClipboard(String(content));
    onCopy?.(content);
    // TODO: close window and show hud thingy
  }, [content, onCopy]);

  return (
    <Action
      title={title}
      icon={icon}
      shortcut={shortcut}
      onAction={handleAction}
    />
  );
};

CopyToClipboard.displayName = "Action.CopyToClipboard";
