import { versionedUrl } from "../asset-version.js";
import { assertWasm64Supported } from "../wasm64.js";
import { state } from "./state.js";

export let Emulator;

export async function ensureWasm() {
  if (state.wasmReady) {
    return;
  }
  assertWasm64Supported();
  const wasmGlue = await import(versionedUrl("../../pkg/emulator.js", import.meta.url).href);
  Emulator = wasmGlue.Emulator;
  state.wasmExports = await wasmGlue.default({
    module_or_path: versionedUrl("../../pkg/emulator_bg.wasm", import.meta.url),
  });
  state.wasmReady = true;
}
