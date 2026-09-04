import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_VM_CORES, VmBooter, jitEnabledForBoot } from "./boot-vm.js?v=20260904-virgl-material-batch-r1";

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

test("multicore saved boots use cooperative JIT while media remains parallel-safe", () => {
  assert.equal(jitEnabledForBoot("media", true, 2), false);
  assert.equal(jitEnabledForBoot("saved-disk", false, 2), true);
});

test("saved disk boot logs stable host and worker phase durations", async (t) => {
  const previousAnimationFrame = globalThis.requestAnimationFrame;
  const previousWorker = globalThis.Worker;
  globalThis.requestAnimationFrame = (callback) => callback();
  globalThis.Worker = FakeWorker;
  FakeWorker.bootRequests = [];
  t.after(() => {
    globalThis.requestAnimationFrame = previousAnimationFrame;
    globalThis.Worker = previousWorker;
  });
  const logs = [];
  let loads = 0;
  let runnerOptions;
  const booter = new VmBooter({
    disk: {
      available: true,
      load: async () => {
        loads += 1;
        return new Uint8Array([1, 2]);
      },
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
    "Fast boot execution setup: 6.8 ms",
    "Fast boot execution mode: cooperative-jit",
  ]);
  assert.deepEqual(runnerOptions, { installedSystem: true, stagedSmp: true });

  await booter.bootInstalledSnapshot(new Uint8Array([3, 4]), "benchmark installed disk", "", false);

  assert.equal(loads, 1, "direct snapshot boot must bypass OPFS");
  assert.equal(logs.at(-4), "OK: installed disk booted");
  assert.equal(FakeWorker.bootRequests.at(-1).payload.extraBootargs, "");
  assert.equal(FakeWorker.bootRequests.at(-1).payload.stagedSmpRequested, false);
});

class FakeWorker {
  static bootRequests = [];

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
    FakeWorker.bootRequests.push(message);
    queueMicrotask(() => {
      const data = {
        id: message.id,
        ok: true,
        value: {
          bootTimings: { firmwarePreparationMs: 12.34, workerPoolMs: 6.78 },
          metrics: bootMetrics(),
          result: "OK: installed disk booted",
          stagedSmp: message.payload.stagedSmpRequested,
        },
      };
      for (const listener of this.listeners) {
        listener({ data });
      }
    });
  }
}

function bootMetrics() {
  return {
    allocatedPages: 1,
    executionMode: "cooperative-jit",
    installDiskAllocatedBytes: 2n,
    installDiskGeneration: 3n,
    installDiskSizeBytes: 4n * 1024n * 1024n * 1024n,
    networkRxPackets: 0n,
    networkStatus: "offline",
    networkTxPackets: 0n,
    networkTxPending: 0,
    pc: 0n,
    totalSteps: 0n,
    uartOutputLen: 0,
  };
}

function sequence(...values) {
  return () => values.shift();
}
