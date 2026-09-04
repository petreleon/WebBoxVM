import assert from "node:assert/strict";
import test, { after } from "node:test";
import { drainNetworkTx, startNetworkProxy, stopNetworkProxy } from "./network.js?v=20260904-virgl-readback-pool-r1";
import { NETWORK_TX_POLL_INTERVAL_MS, state } from "./state.js?v=20260904-virgl-readback-pool-r1";

const previousSelf = globalThis.self;
const previousWebSocket = globalThis.WebSocket;
const previousPerformance = globalThis.performance;
const previousPostMessage = globalThis.postMessage;

globalThis.self = { location: { href: "http://localhost/web/js/vm-worker.js" } };
globalThis.postMessage = () => {};

class FakeWebSocket {
  static OPEN = 1;
  static last;

  readyState = FakeWebSocket.OPEN;
  sent = [];

  constructor() {
    FakeWebSocket.last = this;
    queueMicrotask(() => this.onopen?.());
  }

  send(frame) {
    this.sent.push(frame);
  }

  close() {
    this.readyState = 3;
  }
}

globalThis.WebSocket = FakeWebSocket;

after(() => {
  stopNetworkProxy();
  state.emulator = undefined;
  globalThis.self = previousSelf;
  globalThis.WebSocket = previousWebSocket;
  globalThis.performance = previousPerformance;
  globalThis.postMessage = previousPostMessage;
});

test("idle network drain skips empty tx frame allocation", () => {
  let popped = 0;
  state.emulator = {
    network_tx_pending: () => 0,
    network_tx_frame: () => {
      popped += 1;
      return new Uint8Array();
    },
  };

  startNetworkProxy();

  assert.equal(drainNetworkTx(), 0);
  assert.equal(popped, 0);
});

test("idle network drain throttles pending polls", () => {
  let pendingPolls = 0;
  state.lastNetworkActivityAt = 0;
  state.lastNetworkTxPollAt = 10_000;
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 0;
    },
  };

  startNetworkProxy();

  assert.equal(drainNetworkTx(10_000 + NETWORK_TX_POLL_INTERVAL_MS - 1), 0);
  assert.equal(pendingPolls, 0);
  assert.equal(state.lastNetworkTxPollAt, 10_000);
});

test("recent network activity keeps tx pending polls responsive", () => {
  let pendingPolls = 0;
  state.lastNetworkActivityAt = 2000;
  state.lastNetworkTxPollAt = 2000;
  state.emulator = {
    network_tx_pending: () => {
      pendingPolls += 1;
      return 0;
    },
  };

  assert.equal(drainNetworkTx(2001), 0);
  assert.equal(pendingPolls, 1);
  assert.equal(state.lastNetworkTxPollAt, 2001);
});

test("network drain pops exactly pending frames", () => {
  const frames = [new Uint8Array([1]), new Uint8Array([2])];
  state.emulator = {
    network_tx_pending: () => frames.length,
    network_tx_frame: () => frames.shift() ?? new Uint8Array(),
  };

  assert.equal(drainNetworkTx(), 2);
  assert.deepEqual(
    FakeWebSocket.last.sent.map((frame) => [...frame]),
    [[1], [2]]
  );
});

test("network drain can reuse checked emulator reference", () => {
  const frames = [new Uint8Array([9]), new Uint8Array([10])];
  const emulator = {
    network_tx_pending: () => frames.length,
    network_tx_frame: () => frames.shift() ?? new Uint8Array(),
  };
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return emulator;
    },
  });

  try {
    assert.equal(drainNetworkTx(5000, emulator), 2);
    assert.equal(emulatorReads, 0);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("incoming network frame reuses one checked emulator reference", async () => {
  const previousDescriptor = Object.getOwnPropertyDescriptor(state, "emulator");
  let emulatorReads = 0;
  let injected;
  const emulator = { inject_network_frame: (bytes) => (injected = bytes) };
  Object.defineProperty(state, "emulator", {
    configurable: true,
    get() {
      emulatorReads += 1;
      return emulator;
    },
  });

  try {
    startNetworkProxy();
    await FakeWebSocket.last.onmessage?.({ data: new Uint8Array([11, 12]) });
    assert.deepEqual([...injected], [11, 12]);
    assert.equal(emulatorReads, 1);
  } finally {
    Object.defineProperty(state, "emulator", previousDescriptor);
  }
});

test("network drain reuses caller timestamp for transmitted burst", () => {
  let nowCalls = 0;
  try {
    globalThis.performance = {
      now: () => {
        nowCalls += 1;
        return 100 + nowCalls;
      },
    };
    const frames = [new Uint8Array([3]), new Uint8Array([4])];
    state.lastNetworkActivityAt = 0;
    state.emulator = {
      network_tx_pending: () => frames.length,
      network_tx_frame: () => frames.shift() ?? new Uint8Array(),
    };

    assert.equal(drainNetworkTx(10_000), 2);
    assert.equal(nowCalls, 0);
    assert.equal(state.lastNetworkActivityAt, 10_000);
  } finally {
    globalThis.performance = previousPerformance;
  }
});
