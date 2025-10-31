import Reconciler from "react-reconciler";
import React, { createContext } from "react";

type HostComponent = {
  type: string;
  children: HostComponent[];
};

type Type = string;
interface TextInstance extends HostComponent {
  type: "TEXT";
  text: string;
}
type HostContext = object;
type Instance = HostComponent;
type ChildSet = Array<string | Instance>;
type Timeout = ReturnType<typeof setTimeout>;

type RootContainer = {
  id: "root";
  children: ChildSet;
};

const notImpl = () => {
  throw new Error("Function not implemented.");
};

function serializeReactElement(element: React.ReactElement): unknown {
  if (!element || typeof element !== "object") {
    return element;
  }

  const { type, props } = element;

  const typeName =
    typeof type === "string"
      ? type
      : (type as { displayName?: string }).displayName || "Unknown";

  const serializedProps: Record<string, unknown> = {};

  if (props && typeof props === "object") {
    for (const [key, value] of Object.entries(props)) {
      if (key === "children") {
        continue;
      }

      if (
        value &&
        typeof value === "object" &&
        "type" in value &&
        "props" in value
      ) {
        serializedProps[key] = serializeReactElement(
          value as React.ReactElement
        );
      } else {
        serializedProps[key] = value;
      }
    }
  }

  const children: unknown[] = [];
  const propsWithChildren = props as { children?: React.ReactNode };
  if (propsWithChildren.children) {
    const childArray = React.Children.toArray(propsWithChildren.children);
    for (const child of childArray) {
      if (typeof child === "object" && child && "type" in child) {
        children.push(serializeReactElement(child as React.ReactElement));
      } else if (typeof child === "string" || typeof child === "number") {
        children.push({ type: "TEXT", text: String(child) });
      }
    }
  }

  return {
    type: typeName,
    props: serializedProps,
    children,
  };
}

const HostConfig: Reconciler.HostConfig<
  Type,
  unknown, // Props
  RootContainer,
  Instance,
  TextInstance,
  void, // SuspenseInstance
  void, // HydratableInstance
  void, // FormInstance
  Instance, // PublicInstance
  HostContext,
  ChildSet,
  Timeout, // TimeoutHandle
  -1, // NoTimeout
  void // TransitionStatus
> = {
  supportsPersistence: true,
  supportsMutation: false,
  resolveUpdatePriority: () => 1,
  getCurrentUpdatePriority: () => 1,
  setCurrentUpdatePriority: () => {},
  resolveEventTimeStamp: () => -1.1,
  resolveEventType: () => null,
  trackSchedulerEvent: () => {},
  getRootHostContext: () => ({}),
  getChildHostContext: () => ({}),
  shouldSetTextContent: () => false,
  finalizeInitialChildren: () => false,
  createInstance(type, props, rootContainer, hostContext, internalHandle) {
    let serializedProps: Record<string, unknown> = {};

    if (!!props && typeof props === "object") {
      for (const [key, value] of Object.entries(props)) {
        if (key === "children") continue;

        if (
          value &&
          typeof value === "object" &&
          "type" in value &&
          "props" in value
        ) {
          serializedProps[key] = serializeReactElement(
            value as React.ReactElement
          );
        } else {
          serializedProps[key] = value;
        }
      }
    } else {
      serializedProps = props as Record<string, unknown>;
    }

    return {
      type,
      props: serializedProps,
      children: [],
    };
  },
  cloneInstance(
    instance,
    type,
    oldProps,
    newProps,
    internalInstanceHandle,
    keepChildren,
    recyclableInstance
  ) {
    return instance;
  },
  createTextInstance(text, rootContainer, hostContext, internalHandle) {
    return {
      type: "TEXT",
      text,
      children: [],
    };
  },
  prepareForCommit: () => null,
  resetAfterCommit(containerInfo) {
    console.dir(containerInfo, { depth: null });
    rustyscript.async_functions.updateTree(containerInfo);
  },
  appendInitialChild(parent, child) {
    parent.children.push(child);
  },
  createContainerChildSet(container) {
    return [];
  },
  appendChildToContainerChildSet(childSet, child) {
    childSet.push(child);
  },
  replaceContainerChildren(container, newChildren) {
    container.children = newChildren;
  },
  finalizeContainerChildren(container, newChildren) {
    container.children = newChildren;
  },
  detachDeletedInstance() {},

  getPublicInstance: (instance) => instance,
};

const reconciler = Reconciler(HostConfig);

const root: RootContainer = { id: "root", children: [] };

const container = reconciler.createContainer(
  root,
  0, // LegacyRoot
  null, // hydrationCallbacks
  false, // isStrictMode
  null, // concurrentUpdatesByDefaultOverride

  "", // identifierPrefix
  console.log, // onUncaughtError
  console.log, // onCaughtError
  console.log, // onRecoverableError
  () => {}, // onDefaultTransitionIndicator
  null
);

export const updateContainer = (
  element: React.ReactElement,
  callback?: () => void
) => {
  reconciler.updateContainer(element, container, null, callback);
};

export const batchedUpdates = (callback: () => void) => {
  reconciler.batchedUpdates(callback, null);
};
