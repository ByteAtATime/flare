import { Module } from "module";
import { Console } from "node:console";
import { pack, unpack } from "msgpackr";
import React from "react";
import ReactJsxRuntime from "react/jsx-runtime";
import { NavigationRoot } from "./api/navigation";
import { setPreferences } from "./api/environment";
import { invokeCallback, updateContainer } from "./reconciler";
import { raycastApi } from "./api";
import * as protocol from "./protocol";

const stderrConsole = new Console({
  stdout: process.stderr,
  stderr: process.stderr,
  colorMode: true,
});

const inspectMode = process.env.FLARE_INSPECTOR_MODE === "1";
const profiles = new Map<string, number>();

stderrConsole.timeStamp ??= (label?: string) => {
  if (!inspectMode) return;
  stderrConsole.log(`[${new Date().toISOString()}]${label ? ` ${label}` : ""}`);
};

stderrConsole.profile ??= (label = "default") => {
  if (!inspectMode) return;
  profiles.set(label, performance.now());
};

stderrConsole.profileEnd ??= (label = "default") => {
  if (!inspectMode) return;
  const start = profiles.get(label);
  if (start) {
    console.log(`${label}: ${(performance.now() - start).toFixed(2)}ms`);
    profiles.delete(label);
  }
};

globalThis.console = stderrConsole;

type Request =
  | { type: "initialize"; preferences: Record<string, unknown> }
  | { type: "invokeCallback"; callbackId: string; args: unknown }
  | { type: "pop" }
  | { type: "response"; id: number; result?: unknown; error?: string };

type Response =
  | { type: "initialized"; success: boolean; error?: string }
  | { type: "callbackResult"; success: boolean; error?: string };

const pluginPath = process.argv[2];

if (!pluginPath) {
  console.error("gimme plugin path");
  process.exit(1);
}

const originalRequire = Module.prototype.require;

Module.prototype.require = function (id: string) {
  if (id === "@raycast/api") {
    return raycastApi;
  }
  if (id === "react") {
    return React;
  }
  if (id === "react/jsx-runtime") {
    return ReactJsxRuntime;
  }
  return originalRequire.apply(this, [id]);
};

const sendResponse = (response: Response) => {
  const message = pack(response);
  const length = Buffer.allocUnsafe(4);
  length.writeUInt32BE(message.length, 0);
  process.stdout.write(length);
  process.stdout.write(message);
};

let navigationPopFn: (() => void) | null = null;

export const setNavigationPop = (popFn: () => void) => {
  navigationPopFn = popFn;
};

const initializePlugin = async () => {
  try {
    const pluginModule = require(pluginPath);
    const PluginRoot = pluginModule.default;

    const AppElement = React.createElement(
      NavigationRoot,
      null,
      React.createElement(PluginRoot)
    );

    updateContainer(AppElement, () => {
      sendResponse({ type: "initialized", success: true });
    });
  } catch (error) {
    sendResponse({
      type: "initialized",
      success: false,
      error: String(error),
    });
  }
};

let pluginInitialized = false;

const startCommandLoop = () => {
  let buffer = Buffer.alloc(0);
  let expectedLength: number | null = null;

  process.stdin.on("data", (chunk: Buffer) => {
    buffer = Buffer.concat([buffer, chunk]);

    while (true) {
      if (expectedLength === null) {
        if (buffer.length < 4) break;
        expectedLength = buffer.readUInt32BE(0);
        buffer = buffer.subarray(4);
      }

      if (buffer.length < expectedLength) break;

      const messageData = buffer.subarray(0, expectedLength);
      buffer = buffer.subarray(expectedLength);
      expectedLength = null;

      try {
        const request = unpack(messageData) as Request;

        if (request.type === "initialize") {
          setPreferences(request.preferences);
          if (!pluginInitialized) {
            pluginInitialized = true;
            initializePlugin();
          }
          sendResponse({ type: "callbackResult", success: true });
        } else if (request.type === "invokeCallback") {
          try {
            invokeCallback(request.callbackId, request.args);
            sendResponse({ type: "callbackResult", success: true });
          } catch (error) {
            sendResponse({
              type: "callbackResult",
              success: false,
              error: String(error),
            });
          }
        } else if (request.type === "pop") {
          try {
            if (navigationPopFn) {
              navigationPopFn();
            }
            sendResponse({ type: "callbackResult", success: true });
          } catch (error) {
            sendResponse({
              type: "callbackResult",
              success: false,
              error: String(error),
            });
          }
        } else if (request.type === "response") {
          protocol.handleRustResponse(messageData);
        }
      } catch (error) {
        console.error("something went wrong:", error);
      }
    }
  });
};

startCommandLoop();
