import assert from "node:assert/strict";
import test from "node:test";
import { wasm64ThreadsSupported } from "./wasm64.js?v=20260720-input-latency-r4";

test("threaded wasm64 requires isolation and a shared Memory64", () => {
  class SharedArrayBufferMock {}
  const scope = {
    Atomics: {},
    SharedArrayBuffer: SharedArrayBufferMock,
    WebAssembly: {
      Memory: class {
        constructor(options) {
          assert.equal(options.address, "i64");
          assert.equal(options.shared, true);
          this.buffer = new SharedArrayBufferMock();
        }
      },
    },
    Worker: class {},
    crossOriginIsolated: true,
  };

  assert.equal(wasm64ThreadsSupported(scope), true);
  assert.equal(wasm64ThreadsSupported({ ...scope, crossOriginIsolated: false }), false);
  assert.equal(wasm64ThreadsSupported({ ...scope, Worker: undefined }), false);
});

test("threaded wasm64 rejects a non-shared memory result", () => {
  class SharedArrayBufferMock {}
  const scope = {
    Atomics: {},
    SharedArrayBuffer: SharedArrayBufferMock,
    WebAssembly: { Memory: class { constructor() { this.buffer = {}; } } },
    Worker: class {},
    crossOriginIsolated: true,
  };

  assert.equal(wasm64ThreadsSupported(scope), false);
});
