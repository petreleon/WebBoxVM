import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_VM_CORES, VmBooter, jitEnabledForBoot } from "./boot-vm.js";

test("browser boots default to two virtual CPUs", () => {
  assert.equal(DEFAULT_VM_CORES, 2);
});

test("media boots keep jit disabled by default", () => {
  assert.equal(jitEnabledForBoot("media", false), false);
});

test("media boots honor a manual jit enable", () => {
  assert.equal(jitEnabledForBoot("media", true), true);
});

test("saved disk boots enable jit by default", () => {
  assert.equal(jitEnabledForBoot("saved-disk", false), true);
});

test("multicore boots keep the single-core JIT disabled", () => {
  assert.equal(jitEnabledForBoot("media", true, 2), false);
  assert.equal(jitEnabledForBoot("saved-disk", false, 2), false);
});

test("saved disk boot logs stable host and worker phase durations", async (t) => {
  const previousAnimationFrame = globalThis.requestAnimationFrame;
  const previousWorker = globalThis.Worker;
  globalThis.requestAnimationFrame = (callback) => callback();
  globalThis.Worker = FakeWorker;
  t.after(() => {
    globalThis.requestAnimationFrame = previousAnimationFrame;
    globalThis.Worker = previousWorker;
  });
  const logs = [];
  let runnerOptions;
  const booter = new VmBooter({
    disk: {
      available: true,
      load: async () => new Uint8Array([1, 2]),
      markClean: () => {},
      persistedBytes: 2,
    },
    els: { diskSize: { value: "4" } },
    getJitEnabled: () => false,
    now: sequence(100, 125.04),
    onBooted: () => {},
    resetEmulator: () => {},
    runner: { start: (options) => (runnerOptions = options) },
    setEmulator: () => {},
    term: { clear: () => {}, focus: () => {}, write: () => {} },
    ui: {
      log: (message) => logs.push(message),
      setControls: () => {},
      setStatus: () => {},
      updateMetrics: () => {},
    },
  });

  await booter.bootSavedDisk(Promise.resolve());

  assert.deepEqual(logs, [
    "Wasm64 runtime supported",
    "Fast boot OPFS load: 25.0 ms",
    "OK: installed disk booted",
    "Fast boot firmware preparation: 12.3 ms",
    "Fast boot worker pool: 6.8 ms",
  ]);
  assert.deepEqual(runnerOptions, { installedSystem: true });
});

class FakeWorker {
  constructor() {
    this.listeners = [];
  }

  addEventListener(type, listener) {
    if (type === "message") {
      this.listeners.push(listener);
    }
  }

  postMessage(message) {
    if (message.type !== "bootInstalledDisk") {
      return;
    }
    queueMicrotask(() => {
      const data = {
        id: message.id,
        ok: true,
        value: {
          bootTimings: { firmwarePreparationMs: 12.34, workerPoolMs: 6.78 },
          result: "OK: installed disk booted",
        },
      };
      for (const listener of this.listeners) {
        listener({ data });
      }
    });
  }
}

function sequence(...values) {
  return () => values.shift();
}
