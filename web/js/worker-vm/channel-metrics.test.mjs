import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerChannel } from "./channel.js";

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

test("metrics updates avoid Object.assign allocation", () => {
  const channel = new WorkerChannel("worker.js", callbacks());
  const payload = metrics({ totalSteps: 12n });
  const originalAssign = Object.assign;
  let assignCalls = 0;
  Object.assign = (...args) => {
    assignCalls += 1;
    return originalAssign(...args);
  };

  try {
    FakeWorker.instances[0].emitMessage({ event: "metrics", metrics: payload });
  } finally {
    Object.assign = originalAssign;
  }

  assert.equal(assignCalls, 0);
  assert.equal(channel.metrics.totalSteps, 12n);
});

function callbacks() {
  return { onAutosave() {}, onError() {}, onMetrics() {}, onNetwork() {}, onUart() {} };
}

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
