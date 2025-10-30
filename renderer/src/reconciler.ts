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
  createInstance(type, props, rootContainer, hostContext, internalHandle) {
    console.log("createInstance", type, props);
    return {
      type: type,
      children: [],
    };
  },
  prepareForCommit: () => null,
  resetAfterCommit: () => null,
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
