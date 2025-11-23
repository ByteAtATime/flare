import type * as RaycastApiType from "@raycast/api";
import * as protocol from "../protocol";

class Cache {
  private namespace: string;
  private subscribers: Set<RaycastApiType.Cache.Subscriber>;

  constructor(options?: RaycastApiType.Cache.Options) {
    this.namespace = options?.namespace || "default";
    this.subscribers = new Set();
  }

  public async get(key: string): Promise<string | undefined> {
    const result = await protocol.cacheGet(this.namespace, key);
    return result === null ? undefined : result;
  }

  public async has(key: string): Promise<boolean> {
    return await protocol.cacheHas(this.namespace, key);
  }

  public async set(key: string, data: string): Promise<void> {
    await protocol.cacheSet(this.namespace, key, data);
    this.notifySubscribers(key, data);
  }

  public async remove(key: string): Promise<boolean> {
    const removed = await protocol.cacheRemove(this.namespace, key);
    if (removed) {
      this.notifySubscribers(key, undefined);
    }
    return removed;
  }

  public async clear(
    options: { notifySubscribers: boolean } = { notifySubscribers: true }
  ): Promise<void> {
    await protocol.cacheClear(this.namespace);
    if (options.notifySubscribers) {
      this.notifySubscribers(undefined, undefined);
    }
  }

  public async isEmpty(): Promise<boolean> {
    return await protocol.cacheIsEmpty(this.namespace);
  }

  public subscribe(
    subscriber: RaycastApiType.Cache.Subscriber
  ): RaycastApiType.Cache.Subscription {
    this.subscribers.add(subscriber);
    return () => {
      this.subscribers.delete(subscriber);
    };
  }

  private notifySubscribers(key: string | undefined, data: string | undefined) {
    for (const subscriber of this.subscribers) {
      try {
        subscriber(key, data);
      } catch (e) {
        console.error("Cache subscriber failed", e);
      }
    }
  }
}

export { Cache };
