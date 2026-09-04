import assert from "node:assert/strict";
import test from "node:test";
import {
  drainUart,
  shouldFlushUart,
  shouldPollUart,
} from "./pump.js?v=20260904-virgl-depth-batch-r1";
import {
  UART_POLL_INTERVAL_MS,
  state,
} from "./state.js?v=20260904-virgl-depth-batch-r1";

test("uart flushing batches small bursts for terminal throughput", () => {
  assert.equal(shouldFlushUart(16, 40, 0), false);
  assert.equal(shouldFlushUart(16, 55, 0), true);
});

test("uart flushing sends large chunks immediately", () => {
  assert.equal(shouldFlushUart(8192, 1, 0), true);
});

test("interactive UART bypasses polling and small-output batching", () => {
  assert.equal(shouldPollUart(1001, 1000, true), true);
  assert.equal(shouldFlushUart(1, 1001, 1000, true), true);
});

test("uart polling runs immediately then respects poll cadence", () => {
  assert.equal(shouldPollUart(5, 0), true);
  assert.equal(shouldPollUart(1000 + UART_POLL_INTERVAL_MS - 1, 1000), false);
  assert.equal(shouldPollUart(1000 + UART_POLL_INTERVAL_MS, 1000), true);
});

test("uart drain skips emulator length calls inside poll window", () => {
  let lenCalls = 0;
  state.lastUart = 0;
  state.lastUartPollAt = 1000;
  state.emulator = {
    uart_output_len: () => {
      lenCalls += 1;
      return 0;
    },
  };

  drainUart(1000 + UART_POLL_INTERVAL_MS - 1);

  assert.equal(lenCalls, 0);
  assert.equal(state.lastUartPollAt, 1000);

  state.emulator = undefined;
  state.lastUartPollAt = 0;
});
