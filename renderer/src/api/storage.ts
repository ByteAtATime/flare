import * as protocol from "../protocol";

export const LocalStorage = {
  async getItem<T = string>(key: string): Promise<T | undefined> {
    const result = await protocol.localStorageGet("default", key);
    if (result === null) {
      return undefined;
    }
    try {
      return JSON.parse(result) as T;
    } catch {
      return result as unknown as T;
    }
  },

  async setItem(key: string, value: string | number | boolean): Promise<void> {
    const data =
      typeof value === "string" ? value : JSON.stringify(value);
    await protocol.localStorageSet("default", key, data);
  },

  async removeItem(key: string): Promise<void> {
    await protocol.localStorageRemove("default", key);
  },

  async clear(): Promise<void> {
    await protocol.localStorageClear("default");
  },

  async allItems<T = string>(): Promise<Record<string, T>> {
    const items = await protocol.localStorageAll("default");
    const result: Record<string, T> = {};
    for (const [key, value] of Object.entries(items)) {
      try {
        result[key] = JSON.parse(value) as T;
      } catch {
        result[key] = value as unknown as T;
      }
    }
    return result;
  },
};
