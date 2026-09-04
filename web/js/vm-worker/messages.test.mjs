import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { withEmulatorAccess } from "./access.js?v=20260904-virgl-solid-gpu-readback-r1";
import { handleMessage } from "./messages.js?v=20260904-virgl-solid-gpu-readback-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-solid-gpu-readback-r1";
import { resetUartInput } from "./uart-input.js?v=20260904-virgl-solid-gpu-readback-r1";

afterEach(() => {
  state.emulator = undefined;
  state.executionMode = "cooperative";
  state.jitEnabled = false;
  state.numCores = 0;
  state.parallelTransitionDeferred = false;
  state.pumpScheduled = false;
  state.running = false;
  state.threadedWasm = undefined;
  state.vcpuPool = undefined;
  resetUartInput();
  resetJitState();
});

test("UART input preempts a parallel pump and injects after access is released", async () => {
  let releasePump;
  let markPumpStarted;
  const pumpStarted = new Promise((resolve) => {
    markPumpStarted = resolve;
  });
  const pump = withEmulatorAccess(async () => {
    markPumpStarted();
    await new Promise((resolve) => {
      releasePump = resolve;
    });
  });
  await pumpStarted;
  const calls = [];
  state.emulator = { send_uart_input: (input) => calls.push(["send", input]) };
  state.vcpuPool = { interrupt: () => calls.push(["interrupt"]) };

  const input = handleMessage({ payload: { input: "x" }, type: "sendUartInput" });
  await Promise.resolve();
  assert.deepEqual(calls, [["interrupt"]]);

  releasePump();
  await Promise.all([pump, input]);
  assert.deepEqual(calls, [["interrupt"], ["send", "x"]]);
});

test("UART input remains behind an earlier pause request", async () => {
  let releasePump;
  let markPumpStarted;
  const pumpStarted = new Promise((resolve) => {
    markPumpStarted = resolve;
  });
  const pump = withEmulatorAccess(async () => {
    markPumpStarted();
    await new Promise((resolve) => {
      releasePump = resolve;
    });
  });
  await pumpStarted;
  const calls = [];
  state.running = true;
  state.emulator = {
    send_uart_input(input) {
      calls.push(["send", input, state.running]);
    },
  };
  state.vcpuPool = { interrupt: () => calls.push(["interrupt"]) };

  const pause = handleMessage({ type: "pause" });
  const input = handleMessage({ payload: { input: "x" }, type: "sendUartInput" });
  await Promise.resolve();
  assert.deepEqual(calls, [["interrupt"]]);

  releasePump();
  await Promise.all([pump, pause, input]);

  assert.deepEqual(calls, [["interrupt"], ["send", "x", false]]);
  assert.equal(state.running, false);
  assert.equal(state.urgentUartWaiters, 0);
});
