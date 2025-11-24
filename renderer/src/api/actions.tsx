import { useCallback, type FunctionComponent, type ReactNode } from "react";
import { useNavigation } from "./navigation";
import { Action } from "./components";

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
