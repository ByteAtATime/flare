import { useCallback, type FunctionComponent, type ReactNode } from "react";
import { useNavigation } from "./navigation";
import { Action } from "./components";
import { Clipboard } from "./clipboard";
import { Icon } from "./icons";
import * as protocol from "../protocol";

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
    await Clipboard.copy(content);
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

type OpenInBrowserProps = {
  url: string;
  title?: string;
  icon?: unknown;
  shortcut?: unknown;
  onOpen?: (url: string) => void;
};

export const OpenInBrowser: FunctionComponent<OpenInBrowserProps> = (props) => {
  const {
    url,
    title = "Open in Browser",
    icon = Icon.Globe,
    shortcut,
    onOpen,
  } = props;

  const handleAction = useCallback(async () => {
    await protocol.openUrl(url);
    onOpen?.(url);
  }, [url, onOpen]);

  return (
    <Action
      title={title}
      icon={icon}
      shortcut={shortcut}
      onAction={handleAction}
    />
  );
};

OpenInBrowser.displayName = "Action.OpenInBrowser";
