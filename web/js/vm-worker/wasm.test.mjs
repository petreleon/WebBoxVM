import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { state } from "./state.js?v=20260904-virgl-depth-texture-color-r1";

const originalGlobals = {};

beforeEach(() => {
  for (const name of ["Atomics", "SharedArrayBuffer", "WebAssembly", "Worker", "crossOriginIsolated", "fetch"]) {
    originalGlobals[name] = globalThis[name];
  }
  resetState();
});

afterEach(() => {
  for (const [name, value] of Object.entries(originalGlobals)) {
    if (value === undefined) delete globalThis[name];
    else globalThis[name] = value;
  }
  resetState();
});

test("ensureWasm memoizes concurrent initialization", async () => {
  let fetches = 0;
  installMockRuntime({
    fetch: async () => {
      fetches += 1;
      return response(true);
    },
    threaded: false,
  });
  const { ensureWasm } = await import(`./wasm.js?v=20260904-virgl-depth-texture-color-r1&memoized=${Date.now()}`);

  const first = ensureWasm();
  const second = ensureWasm();
  assert.equal(first, second);
  await Promise.all([first, second]);

  assert.equal(fetches, 1);
  assert.equal(state.wasmReady, true);
  assert.equal(state.threadedWasm, undefined);
});

test("threaded package failure falls back once to the serial package", async () => {
  const urls = [];
  installMockRuntime({
    fetch: async (url) => {
      urls.push(String(url));
      return response(!String(url).includes("pkg-threaded"), 503);
    },
    threaded: true,
  });
  const { ensureWasm } = await import(`./wasm.js?v=20260904-virgl-depth-texture-color-r1&fallback=${Date.now()}`);

  await ensureWasm();

  assert.equal(urls.length, 2);
  assert.match(urls[0], /pkg-threaded/);
  assert.match(urls[1], /\/pkg\//);
  assert.match(state.wasmFallbackReason, /HTTP 503/);
  assert.equal(state.threadedWasm, undefined);
  assert.equal(state.wasmReady, true);
});

function installMockRuntime({ fetch, threaded }) {
  class SharedBuffer {}
  class Memory {
    constructor(options) {
      this.buffer = options.shared ? new SharedBuffer() : {};
    }
  }
  globalThis.Atomics = {};
  globalThis.SharedArrayBuffer = SharedBuffer;
  globalThis.Worker = class {};
  globalThis.crossOriginIsolated = threaded;
  globalThis.fetch = fetch;
  globalThis.WebAssembly = {
    Instance: class {},
    Memory,
    compile: async () => ({}),
    instantiate: async (module) => ({
      instance: { exports: { __wbindgen_start() {}, memory: {} } },
      module,
    }),
  };
}

function resetState() {
  state.threadedWasm = undefined;
  state.wasmExports = undefined;
  state.wasmFallbackReason = undefined;
  state.wasmReady = false;
}

function response(ok, status = 200) {
  return { arrayBuffer: async () => new ArrayBuffer(0), ok, status };
}
