import { versionedUrl } from "../asset-version.js?v=20260904-virgl-depth-compare-r1";
import { assertWasm64Supported, wasm64ThreadsSupported } from "../wasm64.js?v=20260904-virgl-depth-compare-r1";
import { state } from "./state.js?v=20260904-virgl-depth-compare-r1";

export let Emulator;
let wasmInitialization;

export function ensureWasm() {
  if (state.wasmReady) {
    return Promise.resolve();
  }
  wasmInitialization ||= initializeWasm().catch((error) => {
    wasmInitialization = undefined;
    throw error;
  });
  return wasmInitialization;
}

async function initializeWasm() {
  assertWasm64Supported();
  state.wasmFallbackReason = undefined;
  if (wasm64ThreadsSupported()) {
    try {
      const loaded = await loadPackage("../../pkg-threaded");
      requireThreadExports(loaded.glue);
      Emulator = loaded.glue.Emulator;
      state.wasmExports = loaded.exports;
      state.threadedWasm = {
        cancelParallelRun: loaded.glue.cancel_parallel_run,
        finishParallelRun: loaded.glue.finish_parallel_run,
        glueUrl: loaded.glueUrl,
        memory: loaded.exports.memory,
        module: loaded.module,
      };
      state.wasmReady = true;
      return;
    } catch (error) {
      state.wasmFallbackReason = error?.message ?? String(error);
    }
  }
  const loaded = await loadPackage("../../pkg");
  Emulator = loaded.glue.Emulator;
  state.wasmExports = loaded.exports;
  state.threadedWasm = undefined;
  state.wasmReady = true;
}

function requireThreadExports(glue) {
  const required = ["cancel_parallel_run", "finish_parallel_run", "run_parallel_core"];
  const missing = required.find((name) => typeof glue[name] !== "function");
  if (missing) {
    throw new Error(`Threaded Wasm package is missing ${missing}`);
  }
}

async function loadPackage(directory) {
  const glueUrl = versionedUrl(`${directory}/emulator.js`, import.meta.url).href;
  const wasmUrl = versionedUrl(`${directory}/emulator_bg.wasm`, import.meta.url);
  const [glue, response] = await Promise.all([import(glueUrl), fetch(wasmUrl)]);
  if (!response.ok) {
    throw new Error(`Wasm fetch failed with HTTP ${response.status}`);
  }
  const module = await WebAssembly.compile(await response.arrayBuffer());
  const exports = await glue.default({ module_or_path: module });
  return { exports, glue, glueUrl, module };
}
