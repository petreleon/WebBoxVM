import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { VcpuPool } from "./vcpu-pool.js?v=20260903-virgl-capset1-r1";

const previousWorker = globalThis.Worker;

afterEach(() => {
  globalThis.Worker = previousWorker;
});

test("first worker crash cancels the Rust round before joining healthy workers", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      this.core = workers.length;
      this.terminated = false;
      workers.push(this);
    }

    postMessage(message) {
      if (message.type === "init" || message.type === "stop") {
        queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
      } else if (this.core === 0) {
        queueMicrotask(() => this.onerror({ message: "worker crashed" }));
      } else {
        this.runMessage = message;
      }
    }

    terminate() {
      this.terminated = true;
    }
  };
  const calls = [];
  const emulator = {
    parallel_begin_kernel: () => 8n,
  };
  const runtime = threadedWasm({
    cancelParallelRun: (token) => {
      assert.equal(token, 8n);
      calls.push("cancel");
      const message = workers[1].runMessage;
      workers[1].onmessage({ data: { id: message.id, ok: true } });
    },
    finishParallelRun: () => (calls.push("finish"), "cancelled"),
  });
  const pool = await VcpuPool.create(2, runtime);

  await assert.rejects(pool.runRound(emulator, 100), /worker crashed/);
  await assert.rejects(pool.runRound(emulator, 100), /vCPU 0 worker is unavailable/);
  await pool.stop();

  assert.deepEqual(calls, ["cancel", "finish"]);
  assert.ok(workers.every((worker) => worker.terminated));
});

test("request timeout cancels then waits for late worker quiescence", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      this.runMessage = undefined;
      this.terminated = false;
      workers.push(this);
    }
    postMessage(message) {
      if (message.type === "init" || message.type === "stop") {
        queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
      } else {
        this.runMessage = message;
      }
    }
    terminate() {
      this.terminated = true;
    }
  };
  const calls = [];
  const emulator = { parallel_begin_kernel: () => 9n };
  let runResponses = 0;
  const runtime = threadedWasm({
    cancelParallelRun: () => calls.push("cancel"),
    finishParallelRun: () =>
      (assert.equal(runResponses, 2), calls.push("finish"), "cancelled"),
  });
  const options = { requestTimeoutMs: 5, stopTimeoutMs: 5 };
  const pool = await VcpuPool.create(2, runtime, options);
  const round = pool.runRound(emulator, 100);
  const rejected = assert.rejects(round, /request timed out/);
  await new Promise((resolve) => setTimeout(resolve, 15));
  assert.deepEqual(calls, ["cancel"]);
  assert.ok(workers.every((worker) => !worker.terminated));
  const firstStop = pool.stop();
  const secondStop = pool.stop();
  assert.equal(firstStop, secondStop);
  assert.ok(workers.every((worker) => !worker.terminated));
  for (const worker of workers) {
    runResponses += 1;
    worker.onmessage({ data: { id: worker.runMessage.id, ok: true } });
  }
  await rejected;
  await firstStop;
  assert.deepEqual(calls, ["cancel", "finish"]);
  assert.ok(workers.every((worker) => worker.terminated));
});

test("stop terminates workers that do not acknowledge before the deadline", async () => {
  const workers = [];
  globalThis.Worker = class {
    constructor() {
      this.terminated = false;
      workers.push(this);
    }

    postMessage(message) {
      if (message.type === "init") {
        queueMicrotask(() => this.onmessage({ data: { id: message.id, ok: true } }));
      }
    }

    terminate() {
      this.terminated = true;
    }
  };
  const pool = await VcpuPool.create(2, threadedWasm(), { stopTimeoutMs: 5 });

  const firstStop = pool.stop();
  assert.equal(firstStop, pool.stop());
  await firstStop;

  assert.ok(workers.every((worker) => worker.terminated));
});

test("interactive interrupt cancels only the active parallel round", () => {
  const calls = [];
  const pool = new VcpuPool([], threadedWasm({
    cancelParallelRun: (token) => calls.push(token),
  }));

  assert.equal(pool.interrupt(), false);
  pool.activeToken = 12n;
  assert.equal(pool.interrupt(), true);
  assert.equal(pool.interrupt(), true);
  assert.deepEqual(calls, [12n]);
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
