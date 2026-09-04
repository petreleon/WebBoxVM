import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import {
  beginUrgentUartMessage,
  finishUrgentUartMessage,
  injectUartMessage,
  isInputResponsive,
  markUartGuestServiced,
  resetUartInput,
  responsiveStepSlice,
} from "./uart-input.js?v=20260904-virgl-material-batch-r1";
import {
  COOPERATIVE_STEP_SLICE,
  DEFAULT_STEP_SLICE,
  INTERACTIVE_STEP_SLICE,
  INTERACTIVE_WINDOW_MS,
  state,
} from "./state.js?v=20260904-virgl-material-batch-r1";

afterEach(() => {
  resetUartInput();
  state.emulator = undefined;
  state.executionMode = "cooperative";
  state.stepSlice = DEFAULT_STEP_SLICE;
  state.urgentUartWaiters = 0;
  state.vcpuPool = undefined;
});

test("urgent UART input interrupts before FIFO injection", () => {
  const calls = [];
  state.executionMode = "parallel-wasm";
  state.emulator = {
    send_uart_bytes: (input) => calls.push(["bytes", [...input]]),
    send_uart_input: (input) => calls.push(["text", input]),
  };
  state.vcpuPool = { interrupt: () => calls.push(["interrupt"]) };

  const textUrgent = beginUrgentUartMessage({
    payload: { input: "a" },
    type: "sendUartInput",
  });
  const bytesUrgent = beginUrgentUartMessage({
    payload: { input: new Uint8Array([2, 3]) },
    type: "sendUartBytes",
  });
  assert.deepEqual(calls, [["interrupt"], ["interrupt"]]);
  assert.equal(state.urgentUartWaiters, 2);

  assert.equal(injectUartMessage("sendUartInput", "a", state.emulator, 100), true);
  finishUrgentUartMessage(textUrgent);
  assert.equal(
    injectUartMessage("sendUartBytes", new Uint8Array([2, 3]), state.emulator, 101),
    true,
  );
  finishUrgentUartMessage(bytesUrgent);
  assert.deepEqual(calls, [
    ["interrupt"],
    ["interrupt"],
    ["text", "a"],
    ["bytes", [2, 3]],
  ]);
  assert.equal(state.urgentUartWaiters, 0);
  assert.equal(state.uartNeedsGuestService, true);
});

test("responsive slices expire back to throughput mode", () => {
  state.emulator = {};
  state.executionMode = "parallel-wasm";
  state.stepSlice = DEFAULT_STEP_SLICE;
  injectUartMessage("sendUartInput", "x", { send_uart_input() {} }, 100);

  assert.equal(isInputResponsive(100 + INTERACTIVE_WINDOW_MS), true);
  assert.equal(markUartGuestServiced(200), true);
  assert.equal(isInputResponsive(200 + INTERACTIVE_WINDOW_MS - 1), true);
  assert.equal(responsiveStepSlice(200 + INTERACTIVE_WINDOW_MS - 1), INTERACTIVE_STEP_SLICE);
  assert.equal(isInputResponsive(200 + INTERACTIVE_WINDOW_MS), false);
  assert.equal(responsiveStepSlice(200 + INTERACTIVE_WINDOW_MS), DEFAULT_STEP_SLICE);
});

test("cooperative execution stays bounded even outside the interactive window", () => {
  state.executionMode = "cooperative-jit";
  state.stepSlice = DEFAULT_STEP_SLICE;

  assert.equal(responsiveStepSlice(10_000), COOPERATIVE_STEP_SLICE);
});
