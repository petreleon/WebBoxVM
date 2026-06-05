import init, { Emulator } from "../../pkg/emulator.js";
import { assertWasm64Supported } from "../wasm64.js";
import { state } from "./state.js";

export { Emulator };

export async function ensureWasm() {
  if (state.wasmReady) {
    return;
  }
  assertWasm64Supported();
  state.wasmExports = await init({
    module_or_path: new URL("../../pkg/emulator_bg.wasm", import.meta.url),
  });
  state.wasmReady = true;
}
