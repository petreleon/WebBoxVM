import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerChannel } from "./channel.js";

const previousWorker = globalThis.Worker;

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
