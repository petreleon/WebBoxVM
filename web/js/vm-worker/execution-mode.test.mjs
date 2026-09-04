import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import {
  prepareExecutionMode,
  transitionToParallel,
} from "./lifecycle.js?v=20260904-virgl-material-batch-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-material-batch-r1";

afterEach(() => {
  state.executionMode = "cooperative";
  state.jitEnabled = false;
  state.numCores = 0;
  state.parallelTransitionDeferred = false;
  state.pumpScheduled = false;
  state.running = false;
  state.threadedWasm = undefined;
  state.vcpuPool = undefined;
  state.wasmFallbackReason = undefined;
  resetJitState();
});

test("saved multicore boot preflights workers and preserves cooperative jit", async () => {
  const pool = readyPool();
  let createCalls = 0;
  state.jitEnabled = true;
  state.threadedWasm = { module: "threaded" };

  const preparation = await prepareExecutionMode(2, {
    createPool: async () => {
      createCalls += 1;
      return pool;
    },
    deferParallel: true,
  });

  assert.deepEqual(preparation, { bootCores: 2, parallelReady: true });
  assert.equal(state.executionMode, "cooperative-jit");
  assert.equal(state.jitEnabled, true);
  assert.equal(state.numCores, 2);
  assert.equal(state.parallelTransitionDeferred, true);
  assert.equal(state.vcpuPool, pool);

  await transitionToParallel();
  assert.equal(createCalls, 1);
  assert.equal(state.vcpuPool, pool);
});

test("login transition reuses preflighted workers and clears jit state", async () => {
  const pool = readyPool();
  const runtime = { module: "threaded" };
  configureDeferredTransition({ pool, running: true, threadedWasm: runtime });
  state.jitBlockHits.set("1:1000", 1);
  state.jitBlocks.set("1:1000", {});
  state.pumpScheduled = true;

  const result = await transitionToParallel();

  assert.deepEqual(result, { executionMode: "parallel-wasm", transitioned: true });
  assert.equal(state.jitEnabled, false);
  assert.equal(state.jitBlockHits.size, 0);
  assert.equal(state.jitBlocks.size, 0);
  assert.equal(state.running, true);
  assert.equal(state.pumpScheduled, true);
  assert.equal(state.vcpuPool, pool);
});

test("unavailable threaded wasm preserves a two-core cooperative boot", async () => {
  state.executionMode = "cooperative";
  state.jitEnabled = true;

  const result = await prepareExecutionMode(2, { deferParallel: true });

  assert.equal(result.parallelReady, false);
  assert.equal(result.bootCores, 2);
  assert.match(result.reason, /unavailable/i);
  assert.equal(state.executionMode, "cooperative-jit");
  assert.equal(state.jitEnabled, true);
  assert.equal(state.numCores, 2);
  assert.equal(state.parallelTransitionDeferred, false);
  assert.equal(state.vcpuPool, undefined);
});

test("worker preflight failure preserves a two-core cooperative boot", async () => {
  state.jitEnabled = true;
  state.threadedWasm = { module: "threaded" };

  const result = await prepareExecutionMode(2, {
    createPool: async () => {
      throw new Error("worker init failed");
    },
    deferParallel: true,
  });

  assert.deepEqual(result, {
    bootCores: 2,
    parallelReady: false,
    reason: "worker init failed",
  });
  assert.equal(state.executionMode, "cooperative-jit");
  assert.equal(state.numCores, 2);
  assert.equal(state.parallelTransitionDeferred, false);
  assert.equal(state.jitEnabled, true);
  assert.equal(state.vcpuPool, undefined);
});

test("a missing preflight pool cannot activate a second guest CPU mode", async () => {
  configureDeferredTransition({ pool: undefined, running: true, threadedWasm: {} });

  const result = await transitionToParallel();

  assert.equal(result.transitioned, false);
  assert.equal(result.executionMode, "cooperative-jit");
  assert.match(result.reason, /workers are unavailable/i);
  assert.equal(state.running, true);
});

test("an idle preflight worker failure preserves cooperative jit", async () => {
  let stopped = 0;
  const pool = {
    isReady: () => false,
    stop: async () => {
      stopped += 1;
    },
  };
  configureDeferredTransition({ pool, running: true, threadedWasm: {} });

  const result = await transitionToParallel();

  assert.equal(result.transitioned, false);
  assert.equal(result.executionMode, "cooperative-jit");
  assert.equal(stopped, 1);
  assert.equal(state.vcpuPool, undefined);
  assert.equal(state.running, true);
});

function configureDeferredTransition({ pool, running, threadedWasm }) {
  state.executionMode = "cooperative-jit";
  state.jitEnabled = true;
  state.numCores = 2;
  state.parallelTransitionDeferred = true;
  state.running = running;
  state.threadedWasm = threadedWasm;
  state.vcpuPool = pool;
}

function readyPool() {
  return { isReady: () => true, runRound() {} };
}
