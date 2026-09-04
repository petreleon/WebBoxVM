import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { startNetworkProxy, stopNetworkProxy } from "./network.js?v=20260904-virgl-depth-r1";
import { schedulePump } from "./pump.js?v=20260904-virgl-depth-r1";
import { resetJitState, state } from "./state.js?v=20260904-virgl-depth-r1";

const previousPerformance = globalThis.performance;
const previousPostMessage = globalThis.postMessage;
const previousSelf = globalThis.self;
const previousWebSocket = globalThis.WebSocket;

afterEach(() => {
  stopNetworkProxy();
  globalThis.performance = previousPerformance;
  globalThis.postMessage = previousPostMessage;
  globalThis.self = previousSelf;
  globalThis.WebSocket = previousWebSocket;
  state.emulator = undefined;
  resetJitState();
  state.lastAutosavePollAt = 0;
  state.lastMetricsAt = 0;
  state.lastUartPollAt = 0;
  state.networkStatus = "offline";
  state.pumpScheduled = false;
  state.running = false;
});

test("offline pump skips network transmit drain", async () => {
  let pendingPolls = 0;
  let resolveRun;
  const ran = new Promise((resolve) => (resolveRun = resolve));
  class FakeWebSocket {
    static OPEN = 1;
    readyState = FakeWebSocket.OPEN;
    close() {
      this.readyState = 3;
    }
  }
  globalThis.performance = { now: () => 100 };
  globalThis.postMessage = () => {};
  globalThis.self = { location: { href: "http://localhost/web/js/vm-worker.js" } };
  globalThis.WebSocket = FakeWebSocket;
  state.lastAutosavePollAt = 10_000;
  state.lastMetricsAt = 10_000;
  state.lastUartPollAt = 10_000;
  state.running = true;
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 1;
    },
    run_kernel: () => {
      state.running = false;
      resolveRun();
    },
  };

  startNetworkProxy();
  state.networkStatus = "offline";
  schedulePump();
  await ran;

  assert.equal(pendingPolls, 0);
});
