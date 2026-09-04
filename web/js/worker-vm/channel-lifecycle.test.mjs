import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { WorkerChannel } from "./channel.js?v=20260904-virgl-solid-gpu-readback-r1";

const previousWorker = globalThis.Worker;
let worker;

class FakeWorker {
  constructor() {
    this.listeners = new Map();
    worker = this;
  }

  addEventListener(type, listener) {
    this.listeners.set(type, listener);
  }

  emitMessage(data) {
    this.listeners.get("message")?.({ data });
  }

  postMessage(message) {
    this.lastMessage = message;
  }

  terminate() {
    this.terminated = true;
  }
}

beforeEach(() => {
  globalThis.Worker = FakeWorker;
});

afterEach(() => {
  globalThis.Worker = previousWorker;
});

test("free waits for the worker acknowledgment and is idempotent", async () => {
  const channel = new WorkerChannel("worker.js", callbacks(), { freeTimeoutMs: 20 });
  const firstFree = channel.free();

  assert.equal(firstFree, channel.free());
  assert.equal(worker.terminated, undefined);
  assert.equal(worker.lastMessage.type, "free");
  worker.emitMessage({ id: worker.lastMessage.id, ok: true, value: {} });
  await firstFree;

  assert.equal(worker.terminated, true);
  await assert.rejects(channel.request("metrics"), /closing/);
});

test("free terminates an unresponsive worker after its emergency timeout", async () => {
  const channel = new WorkerChannel("worker.js", callbacks(), { freeTimeoutMs: 5 });
  await channel.free();

  assert.equal(worker.lastMessage.type, "free");
  assert.equal(worker.terminated, true);
});

function callbacks() {
  return {
    onAutosave: () => {},
    onError: () => {},
    onMetrics: () => {},
    onUart: () => {},
  };
}
