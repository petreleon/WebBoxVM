import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerChannel } from "./channel.js?v=20260904-virgl-readback-pool-r1";
const previousWorker = globalThis.Worker;
const previousDocument = globalThis.document;
class FakeWorker {
  static instances = [];

  constructor(url, options) {
    this.listeners = new Map();
    this.options = options;
    this.url = url;
    FakeWorker.instances.push(this);
  }

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) ?? [];
    listeners.push(listener);
    this.listeners.set(type, listeners);
  }

  emitMessage(data) {
    for (const listener of this.listeners.get("message") ?? []) {
      listener({ data });
    }
  }

  postMessage(message) {
    this.lastMessage = message;
  }

  terminate() {
    this.terminated = true;
  }
}

beforeEach(() => {
  FakeWorker.instances = [];
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

test("autosave event refreshes cached disk generation before callback", () => {
  let callbackGeneration;
  let channel;
  channel = new WorkerChannel("worker.js", {
    onAutosave: () => {
      callbackGeneration = channel.metrics.installDiskGeneration;
    },
    onError: () => {},
    onMetrics: () => {},
    onNetwork: () => {},
    onUart: () => {},
  });

  FakeWorker.instances[0].emitMessage({
    event: "autosave",
    installDiskGeneration: 42n,
  });

  assert.equal(callbackGeneration, 42n);
  assert.equal(channel.metrics.installDiskGeneration, 42n);
});

test("metrics probe elements are cached after first update", () => {
  const jitProbe = countedProbe();
  const instructionProbe = countedProbe();
  let queryCalls = 0;
  globalThis.document = {
    querySelector(selector) {
      queryCalls += 1;
      if (selector.includes("jit-stats")) {
        return jitProbe;
      }
      if (selector.includes("current-instruction")) {
        return instructionProbe;
      }
      return undefined;
    },
  };
  const channel = new WorkerChannel("worker.js", callbacks());

  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ totalSteps: 1n, currentInstruction: "one" }),
  });
  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ totalSteps: 2n, currentInstruction: "two" }),
  });
  const jitWrites = jitProbe.writeCount;
  const instructionWrites = instructionProbe.writeCount;
  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ totalSteps: 3n, currentInstruction: "two" }),
  });

  assert.equal(queryCalls, 2);
  assert.equal(channel.metrics.totalSteps, 3n);
  assert.equal(jitProbe.textContent, JSON.stringify(channel.metrics.jitStats));
  assert.equal(instructionProbe.textContent, "two");
  assert.equal(jitProbe.writeCount, jitWrites);
  assert.equal(instructionProbe.writeCount, instructionWrites);
});

test("metrics updates keep the cache object stable", () => {
  globalThis.document = { querySelector: () => undefined };
  const channel = new WorkerChannel("worker.js", callbacks());
  const initialMetrics = channel.metrics;

  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ totalSteps: 10n }),
  });
  FakeWorker.instances[0].emitMessage({
    event: "metrics",
    metrics: metrics({ totalSteps: 11n }),
  });

  assert.equal(channel.metrics, initialMetrics);
  assert.equal(channel.metrics.cooperativeIdleFastForwardCycles, 13n);
  assert.equal(channel.metrics.cooperativeWfeParks, 14n);
  assert.equal(channel.metrics.totalSteps, 11n);
});

function callbacks() {
  return {
    onAutosave: () => {},
    onError: () => {},
    onMetrics: () => {},
    onNetwork: () => {},
    onUart: () => {},
  };
}

function countedProbe() {
  let textContent = "";
  let writeCount = 0;
  return {
    get textContent() {
      return textContent;
    },
    set textContent(value) {
      writeCount += 1;
      textContent = value;
    },
    get writeCount() {
      return writeCount;
    },
  };
}

function metrics(overrides = {}) {
  return {
    allocatedPages: 1,
    cooperativeIdleFastForwardCycles: 13n,
    cooperativeWfeParks: 14n,
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
