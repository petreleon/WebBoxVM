import assert from "node:assert/strict";
import test from "node:test";
import { createPumpTaskScheduler } from "./pump.js?v=20260904-virgl-mixed-depth-batch-r1";

test("pump scheduler uses message channel when available", () => {
  let port1;
  class FakeMessageChannel {
    constructor() {
      port1 = { addEventListener: (_, listener) => (port1.listener = listener), start() {} };
      this.port1 = port1;
      this.port2 = { postMessage: () => port1.listener() };
    }
  }
  const scheduler = createPumpTaskScheduler({
    MessageChannelCtor: FakeMessageChannel,
    timeout: () => assert.fail("setTimeout fallback should not run"),
  });
  let ran = false;

  scheduler(() => (ran = true));

  assert.equal(ran, true);
});

test("pump scheduler falls back to timeout without message channel", () => {
  let scheduled;
  let delay;
  const scheduler = createPumpTaskScheduler({
    MessageChannelCtor: null,
    timeout: (callback, ms) => {
      scheduled = callback;
      delay = ms;
    },
  });
  let ran = false;

  scheduler(() => (ran = true));
  scheduled();

  assert.equal(delay, 0);
  assert.equal(ran, true);
});
