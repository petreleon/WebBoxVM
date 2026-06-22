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

test("metrics updates preserve cached jit stats when omitted", () => {
  const channel = new WorkerChannel("worker.js", callbacks());

  FakeWorker.instances[0].emitMessage({ event: "metrics", metrics: metrics() });
  const initialStats = channel.metrics.jitStats;
  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ jitStats: undefined, totalSteps: 13n }),
  });

  assert.equal(channel.metrics.jitStats, initialStats);
  assert.equal(channel.metrics.totalSteps, 13n);
});

test("omitted jit stats skip probe stringify work", () => {
  const jitProbe = { textContent: "" };
  let queryCalls = 0;
  globalThis.document = {
    querySelector(selector) {
      queryCalls += 1;
      return selector.includes("jit-stats") ? jitProbe : undefined;
    },
  };
  const channel = new WorkerChannel("worker.js", callbacks());
  const originalStringify = JSON.stringify;
  let stringifyCalls = 0;
  JSON.stringify = (...args) => {
    stringifyCalls += 1;
    return originalStringify(...args);
  };

  try {
    FakeWorker.instances[0].emitMessage({ event: "metrics", metrics: metrics() });
    FakeWorker.instances[0].emitMessage({
      event: "metrics",
      metrics: metrics({ jitStats: undefined, totalSteps: 13n }),
    });
  } finally {
    JSON.stringify = originalStringify;
  }

  assert.equal(channel.metrics.totalSteps, 13n);
  assert.equal(queryCalls, 3);
  assert.equal(stringifyCalls, 1);
});

test("omitted current instruction skips probe lookup", () => {
  const instructionProbe = { textContent: "" };
  let instructionQueryCalls = 0;
  globalThis.document = {
    querySelector(selector) {
      if (selector.includes("current-instruction")) {
        instructionQueryCalls += 1;
        return instructionProbe;
      }
      return undefined;
    },
  };
  const channel = new WorkerChannel("worker.js", callbacks());

  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ currentInstruction: "one", jitStats: undefined }),
  });
  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metricsWithoutInstruction({ totalSteps: 13n }),
  });

  assert.equal(channel.metrics.currentInstruction, "one");
  assert.equal(channel.metrics.totalSteps, 13n);
  assert.equal(instructionQueryCalls, 1);
});

function callbacks() {
  return { onAutosave() {}, onError() {}, onMetrics() {}, onNetwork() {}, onUart() {} };
}

function metricsWithoutInstruction(overrides = {}) {
  const snapshot = metrics(overrides);
  delete snapshot.currentInstruction;
  return snapshot;
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
