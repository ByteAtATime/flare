import { pack, unpack } from "msgpackr";

type RustRequest =
  | { type: "showToast"; title: string; message?: string; style?: string }
  | { type: "updateTree"; tree: unknown }
  | { type: "cacheSet"; namespace: string; key: string; data: string }
  | { type: "cacheGet"; namespace: string; key: string }
  | { type: "cacheHas"; namespace: string; key: string }
  | { type: "cacheRemove"; namespace: string; key: string }
  | { type: "cacheClear"; namespace: string }
  | { type: "cacheIsEmpty"; namespace: string }
  | { type: "pop" };

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

    const message = pack({ id, ...request });
    const length = Buffer.allocUnsafe(4);
    length.writeUInt32BE(message.length, 0);
    process.stdout.write(length);
    process.stdout.write(message);
  });
};

export const showToast = async (options: {
  title: string;
  message?: string;
  style?: string;
}): Promise<void> => {
  await sendRequest({ type: "showToast", ...options });
};

export const updateTree = async (tree: unknown): Promise<void> => {
  await sendRequest({ type: "updateTree", tree });
};

export const cacheSet = async (
  namespace: string,
  key: string,
  data: string
): Promise<void> => {
  await sendRequest({ type: "cacheSet", namespace, key, data });
};

export const cacheGet = async (
  namespace: string,
  key: string
): Promise<string | null> => {
  const result = await sendRequest({ type: "cacheGet", namespace, key });
  return result as string | null;
};

export const cacheHas = async (
  namespace: string,
  key: string
): Promise<boolean> => {
  const result = await sendRequest({ type: "cacheHas", namespace, key });
  return result as boolean;
};

export const cacheRemove = async (
  namespace: string,
  key: string
): Promise<boolean> => {
  const result = await sendRequest({ type: "cacheRemove", namespace, key });
  return result as boolean;
};

export const cacheClear = async (namespace: string): Promise<void> => {
  await sendRequest({ type: "cacheClear", namespace });
};

export const cacheIsEmpty = async (namespace: string): Promise<boolean> => {
  const result = await sendRequest({ type: "cacheIsEmpty", namespace });
  return result as boolean;
};

export const pop = async (): Promise<void> => {
  await sendRequest({ type: "pop" });
};

export const handleRustResponse = (data: Buffer) => {
  try {
    const response = unpack(data) as RustResponse & { id: number };
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
