import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerVm } from "./worker-vm.js";

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

  postMessage() {}
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

function metrics(overrides = {}) {
  return {
    allocatedPages: 1,
    currentInstruction: undefined,
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
