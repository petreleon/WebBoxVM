import assert from "node:assert/strict";
import test from "node:test";
import { BootParallelTransition } from "./boot-parallel-transition.js?v=20260718-staged-fast-boot";

test("CPU1 and login milestones request and report one parallel transition", async () => {
  let calls = 0;
  let resolveLogged;
  const logged = new Promise((resolve) => {
    resolveLogged = resolve;
  });
  const logs = [];
  const emulator = {
    transition_to_parallel: async () => {
      calls += 1;
      return { executionMode: "parallel-wasm", transitioned: true };
    },
  };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: assert.fail,
    ui: {
      log(message) {
        logs.push(message);
        resolveLogged();
      },
      updateMetrics: () => {},
    },
  });

  controller.observe({ name: "cpu1-online" });
  controller.observe({ name: "login-prompt" });
  controller.observe({ name: "login-prompt" });
  await logged;

  assert.equal(calls, 1);
  assert.deepEqual(logs, ["Fast boot execution mode: parallel-wasm"]);
});

test("login alone keeps the safe cooperative mode", async () => {
  let calls = 0;
  const emulator = {
    transition_to_parallel: async () => {
      calls += 1;
    },
  };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: assert.fail,
    ui: { log: assert.fail, updateMetrics: assert.fail },
  });

  controller.observe({ name: "login-prompt" });
  await Promise.resolve();

  assert.equal(calls, 0);
});

test("failed parallel transition is reported as a cooperative-jit fallback", async () => {
  let resolveLogged;
  const logged = new Promise((resolve) => {
    resolveLogged = resolve;
  });
  const logs = [];
  const emulator = {
    transition_to_parallel: async () => ({
      executionMode: "cooperative-jit",
      reason: "threads unavailable",
      transitioned: false,
    }),
  };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: assert.fail,
    ui: {
      log(message) {
        logs.push(message);
        resolveLogged();
      },
      updateMetrics: () => {},
    },
  });

  controller.observe({ name: "login-prompt" });
  controller.observe({ name: "cpu1-online" });
  await logged;

  assert.deepEqual(logs, [
    "Fast boot execution mode: cooperative-jit (threads unavailable)",
  ]);
});

test("reset invalidates an in-flight transition failure", async () => {
  let rejectTransition;
  const transition = new Promise((_resolve, reject) => {
    rejectTransition = reject;
  });
  let errors = 0;
  const emulator = { transition_to_parallel: () => transition };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: () => {
      errors += 1;
    },
    ui: { log: assert.fail, updateMetrics: assert.fail },
  });

  controller.observe({ name: "cpu1-online" });
  controller.observe({ name: "login-prompt" });
  await Promise.resolve();
  controller.reset();
  rejectTransition(new Error("stale worker failure"));
  await transition.catch(() => {});
  await Promise.resolve();

  assert.equal(errors, 0);
});

test("reset invalidates an in-flight transition result", async () => {
  let resolveTransition;
  const transition = new Promise((resolve) => {
    resolveTransition = resolve;
  });
  let reports = 0;
  const emulator = { transition_to_parallel: () => transition };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: assert.fail,
    ui: {
      log: () => {
        reports += 1;
      },
      updateMetrics: () => {
        reports += 1;
      },
    },
  });

  controller.observe({ name: "cpu1-online" });
  controller.observe({ name: "login-prompt" });
  await Promise.resolve();
  controller.reset();
  resolveTransition({ executionMode: "parallel-wasm", transitioned: true });
  await transition;
  await Promise.resolve();

  assert.equal(reports, 0);
});

test("reset cancels a transition before its first microtask", async () => {
  let calls = 0;
  const emulator = {
    transition_to_parallel: () => {
      calls += 1;
    },
  };
  const controller = new BootParallelTransition({
    disk: {},
    getEmulator: () => emulator,
    handleError: assert.fail,
    ui: { log: assert.fail, updateMetrics: assert.fail },
  });

  controller.observe({ name: "cpu1-online" });
  controller.observe({ name: "login-prompt" });
  controller.reset();
  await Promise.resolve();
  await Promise.resolve();

  assert.equal(calls, 0);
});
