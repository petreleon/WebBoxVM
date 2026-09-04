import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { VcpuPool } from "./vcpu-pool.js?v=20260904-virgl-depth-batch-compare-r1";

const previousWorker = globalThis.Worker;

afterEach(() => {
  globalThis.Worker = previousWorker;
});

test("vcpu pool initializes every worker and completes one bounded round", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      this.messages = [];
      this.terminated = false;
      workers.push(this);
    }
    postMessage(message) {
      this.messages.push(message);
      queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true, value: {} } }));
    }
    terminate() {
      this.terminated = true;
    }
  };
  const calls = [];
  const emulator = {
    parallel_begin_kernel: (steps) => (calls.push(["begin", steps]), 7n),
  };
  const pool = await VcpuPool.create(2, threadedWasm({
    finishParallelRun: (token) => (assert.equal(token, 7n), calls.push(["finish"]), "done"),
  }));

  assert.equal(pool.isReady(2), true);
  assert.equal(await pool.runRound(emulator, 50), "done");
  await pool.stop();

  assert.equal(pool.isReady(2), false);
  assert.deepEqual(calls, [["begin", 50], ["finish"]]);
  assert.deepEqual(
    workers.map((worker) => worker.messages.map((message) => message.type)),
    [["init", "run", "stop"], ["init", "run", "stop"]],
  );
  assert.ok(workers.every((worker) => worker.messages[1].token === 7n));
  assert.ok(workers.every((worker) => worker.terminated));
});

test("partial worker initialization failure tears down the entire pool", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      this.core = workers.length;
      this.messages = [];
      this.terminated = false;
      workers.push(this);
    }
    postMessage(message) {
      this.messages.push(message);
      if (message.type === "stop" || this.core === 0) {
        queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
      } else {
        queueMicrotask(() => this.onerror({ message: "worker init failed" }));
      }
    }
    terminate() {
      this.terminated = true;
    }
  };

  await assert.rejects(
    VcpuPool.create(2, threadedWasm(), { stopTimeoutMs: 5 }),
    /worker init failed/,
  );

  assert.deepEqual(
    workers.map((worker) => worker.messages.map((message) => message.type)),
    [["init", "stop"], ["init"]],
  );
  assert.ok(workers.every((worker) => worker.terminated));
});

test("worker constructor failure tears down earlier workers", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      if (workers.length === 1) {
        throw new Error("worker constructor failed");
      }
      this.messages = [];
      this.terminated = false;
      workers.push(this);
    }
    postMessage(message) {
      this.messages.push(message);
      queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
    }
    terminate() {
      this.terminated = true;
    }
  };

  await assert.rejects(
    VcpuPool.create(2, threadedWasm(), { stopTimeoutMs: 5 }),
    /worker constructor failed/,
  );

  assert.deepEqual(workers[0].messages.map((message) => message.type), ["stop"]);
  assert.equal(workers[0].terminated, true);
});

function threadedWasm(overrides = {}) {
  return {
    cancelParallelRun: () => {},
    finishParallelRun: () => "done",
    glueUrl: "http://localhost/pkg-threaded/emulator.js",
    memory: {},
    module: {},
    ...overrides,
  };
}
