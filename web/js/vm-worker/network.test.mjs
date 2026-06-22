import assert from "node:assert/strict";
import test, { after } from "node:test";
import { drainNetworkTx, startNetworkProxy, stopNetworkProxy } from "./network.js";
import { state } from "./state.js";

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

test("network drain marks activity once per transmitted burst", () => {
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

    assert.equal(drainNetworkTx(), 2);
    assert.equal(nowCalls, 1);
    assert.equal(state.lastNetworkActivityAt, 101);
  } finally {
    globalThis.performance = previousPerformance;
  }
});
