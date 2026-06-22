import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerChannel } from "./channel.js";

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
  const jitProbe = { textContent: "" };
  const instructionProbe = { textContent: "" };
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

  assert.equal(queryCalls, 2);
  assert.equal(channel.metrics.totalSteps, 2n);
  assert.equal(jitProbe.textContent, JSON.stringify(channel.metrics.jitStats));
  assert.equal(instructionProbe.textContent, "two");
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
