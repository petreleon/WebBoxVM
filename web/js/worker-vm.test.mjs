import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerVm } from "./worker-vm.js?v=20260903-virgl-viewport-r1";

const previousDocument = globalThis.document;
const previousWorker = globalThis.Worker;

class FakeWorker {
  static instances = [];

  constructor() {
    this.listeners = new Map();
    FakeWorker.instances.push(this);
  }

  addEventListener(type, listener) {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener]);
  }

  emitMessage(data) {
    for (const listener of this.listeners.get("message") ?? []) {
      listener({ data });
    }
  }

  postMessage(message) {
    this.lastMessage = message;
  }
  terminate() {}
}

beforeEach(() => {
  FakeWorker.instances = [];
  globalThis.document = { querySelector: () => undefined };
  globalThis.Worker = FakeWorker;
});

afterEach(() => {
  globalThis.Worker = previousWorker;
  if (previousDocument === undefined) {
    delete globalThis.document;
  } else {
    globalThis.document = previousDocument;
  }
});

test("network stats reuse a stable view while reflecting updates", () => {
  const vm = new WorkerVm();

  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ networkRxPackets: 1n, networkTxPackets: 2n }),
  });
  const first = vm.network_stats();

  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ networkRxPackets: 3n, networkTxPackets: 4n, networkTxPending: 5 }),
  });
  const second = vm.network_stats();

  assert.equal(second, first);
  assert.deepEqual(first, {
    rxPackets: 3n,
    status: "connected",
    txPackets: 4n,
    txPending: 5,
  });
});

test("installed disk boot retains worker phase timings", async () => {
  const vm = new WorkerVm();
  const boot = vm.boot_installed_disk(new Uint8Array([1, 2]), 2);
  const worker = FakeWorker.instances[0];

  worker.emitMessage({
    id: worker.lastMessage.id,
    ok: true,
    value: {
      bootTimings: { firmwarePreparationMs: 12.5, workerPoolMs: 3.5 },
      result: "OK: booted",
      stagedSmp: true,
    },
  });

  assert.equal(await boot, "OK: booted");
  assert.deepEqual(vm.boot_timings(), {
    firmwarePreparationMs: 12.5,
    workerPoolMs: 3.5,
  });
  assert.equal(vm.staged_smp_enabled(), true);
});

test("installed disk boot serializes an explicit staged SMP opt-out", async () => {
  const vm = new WorkerVm();
  const boot = vm.boot_installed_disk(new Uint8Array([1, 2]), 2, "", false);
  const worker = FakeWorker.instances[0];

  assert.equal(worker.lastMessage.type, "bootInstalledDisk");
  assert.equal(worker.lastMessage.payload.extraBootargs, "");
  assert.equal(worker.lastMessage.payload.stagedSmpRequested, false);
  worker.emitMessage({
    id: worker.lastMessage.id,
    ok: true,
    value: { bootTimings: {}, result: "OK: booted", stagedSmp: false },
  });

  assert.equal(await boot, "OK: booted");
  assert.equal(vm.staged_smp_enabled(), false);
});

test("parallel transition uses a request so callers can observe completion", async () => {
  const vm = new WorkerVm();
  const transition = vm.transition_to_parallel();
  const worker = FakeWorker.instances[0];

  assert.equal(worker.lastMessage.type, "transitionToParallel");
  worker.emitMessage({
    id: worker.lastMessage.id,
    ok: true,
    value: {
      executionMode: "parallel-wasm",
      metrics: metrics({ executionMode: "parallel-wasm" }),
      transitioned: true,
    },
  });

  assert.deepEqual(await transition, {
    executionMode: "parallel-wasm",
    transitioned: true,
  });
  assert.equal(vm.execution_mode(), "parallel-wasm");
});

test("GPU events and 3D acknowledgments preserve transferable packets", () => {
  const vm = new WorkerVm();
  const packet = new Uint8Array([1, 2, 3]);
  const received = [];
  vm.onGpuFrame = (value) => received.push(["2d", value]);
  vm.onGpu3dFrame = (value) => received.push(["3d", value]);
  vm.onGpuReset = (value) => received.push(["reset", value]);

  FakeWorker.instances[0].emitMessage({ event: "gpuFrame", packet });
  FakeWorker.instances[0].emitMessage({ event: "gpu3dFrame", packet });
  FakeWorker.instances[0].emitMessage({ event: "gpuReset", generation: 4 });
  vm.gpu3d_ack(17, true);

  assert.deepEqual(received, [["2d", packet], ["3d", packet], ["reset", 4]]);
  assert.deepEqual(FakeWorker.instances[0].lastMessage, {
    payload: { sequence: 17, success: true },
    type: "gpu3dAck",
  });
});

function metrics(overrides = {}) {
  return {
    allocatedPages: 1,
    currentInstruction: undefined,
    executionMode: "cooperative",
    installDiskAllocatedBytes: 2n,
    installDiskGeneration: 3n,
    installDiskSizeBytes: 4n,
    jitStats: { cacheBlocks: 1, enabled: true, hitSites: 1 },
    networkRxPackets: 5n,
    networkStatus: "connected",
    networkTxPackets: 6n,
    networkTxPending: 0,
    pc: 7n,
    totalSteps: 8n,
    uartOutputLen: 9,
    ...overrides,
  };
}
