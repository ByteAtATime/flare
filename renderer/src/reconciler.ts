import Reconciler from "react-reconciler";
import React from "react";
import * as protocol from "./protocol";

type HostComponent = {
  type: string;
  props: Record<string, unknown>;
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

const callbackRegistry = new Map<string, Function>();
let callbackIdCounter = 0;

const registerCallback = (callback: Function) => {
  const id = `callback_${callbackIdCounter++}`;
  callbackRegistry.set(id, callback);
  return {
    type: "CALLBACK",
    id,
  };
};

export const invokeCallback = async (id: string, args: any) => {
  const callback = callbackRegistry.get(id);
  if (callback) {
    batchedUpdates(() => {
      callback(args);
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
  } else {
    console.warn(`No callback found for id: ${id}`);
  }
};

function processProps(props: Record<string, unknown>): Record<string, unknown> {
  const processed: Record<string, unknown> = {};

  if (!props) return processed;

  for (const [key, value] of Object.entries(props)) {
    if (key === "children") continue;

    if (typeof value === "function") {
      processed[key] = registerCallback(value);
    } else {
      processed[key] = value;
    }
  }
  return processed;
}

const HostConfig: Reconciler.HostConfig<
  Type,
  Record<string, unknown>, // Props
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
    if (type === "flare-nav-stack") {
      return {
        type: "flare-nav-stack",
        props: {},
        children: [],
      };
    }

    if (type === "flare-slot") {
      return {
        type: "flare-slot",
        props: props as Record<string, unknown>,
        children: [],
      };
    }

    return {
      type,
      props: processProps(props),
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
    return {
      type,
      props: processProps(newProps),
      children: keepChildren ? instance.children : [],
    };
  },

  createTextInstance(text, rootContainer, hostContext, internalHandle) {
    return {
      type: "TEXT",
      props: {},
      text,
      children: [],
    };
  },

  prepareForCommit: () => null,

  resetAfterCommit(node) {
    const navStackNode = node.children[0];

    if (!navStackNode) {
      return;
    }

    if (
      typeof navStackNode !== "object" ||
      navStackNode?.type !== "flare-nav-stack"
    ) {
      throw new Error("Expected root child to be a flare-nav-stack.", {
        cause: { node },
      });
    }

    const screens = navStackNode.children;
    const activeComponent = screens[screens.length - 1];

    protocol.updateTree({
      id: "root",
      children: [activeComponent],
    });
  },

  appendInitialChild(parent, child) {
    if (child.type === "flare-slot") {
      const slotName = child.props.name as string;
      if (child.children.length > 0) {
        parent.props[slotName] = child.children[0];
      }
    } else {
      parent.children.push(child);
    }
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

  scheduleTimeout: setTimeout,
  cancelTimeout: clearTimeout,
  noTimeout: -1,

  maySuspendCommit: () => false,
  preloadInstance: () => true,
  startSuspendingCommit: () => {},
  suspendInstance: () => {},
  waitForCommitToBeReady: () => null,
};

const reconciler = Reconciler(HostConfig);

const root: RootContainer = { id: "root", children: [] };

const container = reconciler.createContainer(
  root,
  1, // ConcurrentRoot
  null, // hydrationCallbacks
  false, // isStrictMode
  null, // concurrentUpdatesByDefaultOverride

  "", // identifierPrefix
  console.error, // onUncaughtError
  console.error, // onCaughtError
  console.error, // onRecoverableError
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
