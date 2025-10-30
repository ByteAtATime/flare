import Reconciler from "react-reconciler";

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
    const serializedProps =
      !!props && typeof props === "object" && "children" in props
        ? Object.fromEntries(
            Object.entries(props).filter(([key]) => key !== "children")
          )
        : props;
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
