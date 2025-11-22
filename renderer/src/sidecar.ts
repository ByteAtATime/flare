import { Module } from "module";
import { Console } from "node:console";
import {
  React,
  NavigationRoot,
  updateContainer,
  invokeCallback,
  ReactJsxRuntime,
  raycastApi,
} from "./index";
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
  | { type: "initialize"; pluginPath: string }
  | { type: "invokeCallback"; callbackId: string; args: unknown }
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
  process.stdout.write(JSON.stringify(response) + "\n");
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

const startCommandLoop = () => {
  let buffer = "";

  process.stdin.on("data", (chunk) => {
    buffer += chunk.toString();

    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (!line.trim()) continue;

      try {
        const request = JSON.parse(line) as Request;

        if (request.type === "invokeCallback") {
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
        } else if (request.type === "response") {
          protocol.handleRustResponse(line);
        }
      } catch (error) {
        console.error("something went wrong:", error);
      }
    }
  });
};

startCommandLoop();
initializePlugin();
