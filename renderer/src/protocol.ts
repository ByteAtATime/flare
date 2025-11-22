type RustRequest =
  | { type: "showToast"; title: string; message?: string; style?: string }
  | { type: "updateTree"; tree: unknown }
  | { type: "cacheSet"; namespace: string; key: string; data: string }
  | { type: "cacheGet"; namespace: string; key: string }
  | { type: "cacheHas"; namespace: string; key: string }
  | { type: "cacheRemove"; namespace: string; key: string }
  | { type: "cacheClear"; namespace: string }
  | { type: "cacheIsEmpty"; namespace: string };

type RustResponse =
  | { type: "success"; result?: unknown }
  | { type: "error"; error: string };

let requestId = 0;
const pendingRequests = new Map<
  number,
  { resolve: (value: unknown) => void; reject: (error: Error) => void }
>();

const sendRequest = (request: RustRequest): Promise<unknown> => {
  return new Promise((resolve, reject) => {
    const id = requestId++;
    pendingRequests.set(id, { resolve, reject });

    const message = JSON.stringify({ id, ...request });
    process.stdout.write(message + "\n");
  });
};

export const rustyscript = {
  async_functions: {
    showToast: async (options: {
      title: string;
      message?: string;
      style?: string;
    }) => {
      await sendRequest({ type: "showToast", ...options });
    },
    updateTree: async (tree: unknown) => {
      await sendRequest({ type: "updateTree", tree });
    },
  },
  functions: {
    cacheSet: (namespace: string, key: string, data: string) => {
      sendRequest({ type: "cacheSet", namespace, key, data });
    },
    cacheGet: (namespace: string, key: string): string | null => {
      throw new Error("Sync cache operations not supported yet");
    },
    cacheHas: (namespace: string, key: string): boolean => {
      throw new Error("Sync cache operations not supported yet");
    },
    cacheRemove: (namespace: string, key: string): boolean => {
      throw new Error("Sync cache operations not supported yet");
    },
    cacheClear: (namespace: string) => {
      throw new Error("Sync cache operations not supported yet");
    },
    cacheIsEmpty: (namespace: string): boolean => {
      throw new Error("Sync cache operations not supported yet");
    },
  },
};

export const handleRustResponse = (line: string) => {
  try {
    const response = JSON.parse(line) as RustResponse & { id: number };
    const pending = pendingRequests.get(response.id);

    if (pending) {
      pendingRequests.delete(response.id);

      if (response.type === "success") {
        pending.resolve(response.result);
      } else {
        pending.reject(new Error(response.error));
      }
    }
  } catch (error) {
    console.error("Failed to handle Rust response:", error);
  }
};
