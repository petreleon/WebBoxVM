import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { drainUart } from "./pump.js?v=20260904-virgl-solid-batch-r1";
import { state } from "./state.js?v=20260904-virgl-solid-batch-r1";

const previousPostMessage = globalThis.postMessage;

afterEach(() => {
  globalThis.postMessage = previousPostMessage;
  state.emulator = undefined;
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
});

test("uart drain can reuse checked emulator reference", () => {
  const messages = [];
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  const emulator = {
    uart_output_len: () => 8192,
    uart_output_since: () => "x".repeat(8192),
  };
  globalThis.postMessage = (message) => messages.push(message);
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return emulator;
    },
  });

  try {
    assert.equal(drainUart(10, emulator), true);

    assert.equal(emulatorReads, 0);
    assert.equal(messages.length, 1);
    assert.equal(messages[0].output.length, 8192);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});
