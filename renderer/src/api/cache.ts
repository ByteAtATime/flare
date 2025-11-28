import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import os from "node:os";
import type * as RaycastApiType from "@raycast/api";

export class Cache {
  private directory: string;
  private subscribers: Set<RaycastApiType.Cache.Subscriber>;

  constructor(options?: RaycastApiType.Cache.Options) {
    const namespace = options?.namespace ?? "default";
    const cacheRoot =
      process.env.XDG_CACHE_HOME ?? path.join(os.homedir(), ".cache");
    this.directory = path.join(cacheRoot, "flare", namespace);
    this.subscribers = new Set();

    if (!fs.existsSync(this.directory)) {
      fs.mkdirSync(this.directory, { recursive: true });
    }
  }

  private getPath(key: string): string {
    const hash = crypto.createHash("sha256").update(key).digest("hex");
    return path.join(this.directory, hash);
  }

  public get(key: string): string | undefined {
    const filePath = this.getPath(key);
    if (fs.existsSync(filePath)) {
      return fs.readFileSync(filePath, "utf-8");
    }
    return undefined;
  }

  public has(key: string): boolean {
    return fs.existsSync(this.getPath(key));
  }

  public set(key: string, data: string): void {
    const filePath = this.getPath(key);
    fs.writeFileSync(filePath, data);
    this.notifySubscribers(key, data);
  }

  public remove(key: string): boolean {
    const filePath = this.getPath(key);
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath);
      this.notifySubscribers(key, undefined);
      return true;
    }
    return false;
  }

  public clear(
    options: { notifySubscribers: boolean } = { notifySubscribers: true }
  ): void {
    if (fs.existsSync(this.directory)) {
      const files = fs.readdirSync(this.directory);
      for (const file of files) {
        fs.unlinkSync(path.join(this.directory, file));
      }
    }
    if (options.notifySubscribers) {
      this.notifySubscribers(undefined, undefined);
    }
  }

  public isEmpty(): boolean {
    if (!fs.existsSync(this.directory)) {
      return true;
    }
    const files = fs.readdirSync(this.directory);
    return files.length === 0;
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
