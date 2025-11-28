import { pack, Packr, unpack } from "msgpackr";

export type ClipboardContent =
  | { text: string }
  | { file: string }
  | { html: string; text?: string };

type RustRequest =
  | { type: "showToast"; title: string; message?: string; style?: string }
  | { type: "updateTree"; tree: unknown }
  | { type: "localStorageSet"; namespace: string; key: string; data: string }
  | { type: "localStorageGet"; namespace: string; key: string }
  | { type: "localStorageRemove"; namespace: string; key: string }
  | { type: "localStorageClear"; namespace: string }
  | { type: "localStorageAll"; namespace: string }
  | { type: "pop" }
  | { type: "openExtensionPreferences" }
  | { type: "openCommandPreferences" }
  | { type: "clipboardCopy"; content: ClipboardContent; concealed: boolean }
  | { type: "clipboardClear" }
  | { type: "clipboardRead"; offset?: number }
  | { type: "openUrl"; url: string }
  | { type: "oauthAuthorize"; url: string; state: string }
  | { type: "oauthSetTokens"; providerId: string; tokens: string }
  | { type: "oauthGetTokens"; providerId: string }
  | { type: "oauthRemoveTokens"; providerId: string };

type RustResponse =
  | { type: "success"; result?: unknown }
  | { type: "error"; error: string };

let requestId = 0;
const pendingRequests = new Map<
  number,
  { resolve: (value: unknown) => void; reject: (error: Error) => void }
>();

const packr = new Packr({
  // msgpackr tries to optimize small maps into structs by default,
  // but rmp_serde doesn't understand that
  // TODO: does this help performance, and if so, can we implement it in rmp_serde?
  useRecords: false,

  // it also tries to encode undefined as a custom extension
  encodeUndefinedAsNil: true,
});

const sendRequest = (request: RustRequest): Promise<unknown> => {
  return new Promise((resolve, reject) => {
    const id = requestId++;
    pendingRequests.set(id, { resolve, reject });

    const message = packr.pack({ id, ...request });
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

export const localStorageSet = async (
  namespace: string,
  key: string,
  data: string
): Promise<void> => {
  await sendRequest({ type: "localStorageSet", namespace, key, data });
};

export const localStorageGet = async (
  namespace: string,
  key: string
): Promise<string | null> => {
  const result = await sendRequest({ type: "localStorageGet", namespace, key });
  return result as string | null;
};

export const localStorageRemove = async (
  namespace: string,
  key: string
): Promise<boolean> => {
  const result = await sendRequest({
    type: "localStorageRemove",
    namespace,
    key,
  });
  return result as boolean;
};

export const localStorageClear = async (namespace: string): Promise<void> => {
  await sendRequest({ type: "localStorageClear", namespace });
};

export const localStorageAll = async (
  namespace: string
): Promise<Record<string, string>> => {
  const result = await sendRequest({ type: "localStorageAll", namespace });
  return result as Record<string, string>;
};

export const pop = async (): Promise<void> => {
  await sendRequest({ type: "pop" });
};

export const openExtensionPreferences = async (): Promise<void> => {
  await sendRequest({ type: "openExtensionPreferences" });
};

export const openCommandPreferences = async (): Promise<void> => {
  await sendRequest({ type: "openCommandPreferences" });
};

export const clipboardCopy = async (
  content: ClipboardContent,
  concealed: boolean
): Promise<void> => {
  await sendRequest({ type: "clipboardCopy", content, concealed });
};

export const clipboardClear = async (): Promise<void> => {
  await sendRequest({ type: "clipboardClear" });
};

export const clipboardRead = async (
  offset?: number
): Promise<{ text: string; html?: string; file?: string }> => {
  const result = await sendRequest({ type: "clipboardRead", offset });
  return result as { text: string; html?: string; file?: string };
};

export const openUrl = async (url: string): Promise<void> => {
  await sendRequest({ type: "openUrl", url });
};

export const oauthAuthorize = async (
  url: string,
  state: string
): Promise<{ authorizationCode: string }> => {
  const result = await sendRequest({ type: "oauthAuthorize", url, state });
  return result as { authorizationCode: string };
};

export const oauthSetTokens = async (
  providerId: string,
  tokens: string
): Promise<void> => {
  await sendRequest({ type: "oauthSetTokens", providerId, tokens });
};

export const oauthGetTokens = async (
  providerId: string
): Promise<string | null> => {
  const result = await sendRequest({ type: "oauthGetTokens", providerId });
  return result as string | null;
};

export const oauthRemoveTokens = async (providerId: string): Promise<void> => {
  await sendRequest({ type: "oauthRemoveTokens", providerId });
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
