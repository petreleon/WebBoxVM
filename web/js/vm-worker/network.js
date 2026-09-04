import { NETWORK_IDLE_FAST_MS, NETWORK_TX_POLL_INTERVAL_MS, state } from "./state.js?v=20260904-virgl-depth-texture-r1";
import { withEmulatorAccess } from "./access.js?v=20260904-virgl-depth-texture-r1";

const RETRY_MS = 3000;
let socket;
let retryTimer;

export function startNetworkProxy() {
  if (socket || retryTimer) {
    return;
  }
  connect();
}

export function stopNetworkProxy() {
  clearRetry();
  if (socket) {
    socket.onclose = null;
    socket.close();
    socket = undefined;
  }
  setStatus("offline");
}

export function drainNetworkTx(now = performance.now(), emulator = state.emulator) {
  if (!emulator || !socket || socket.readyState !== WebSocket.OPEN) {
    return 0;
  }
  if (!shouldPollNetworkTx(now)) {
    return 0;
  }
  state.lastNetworkTxPollAt = now;
  const pending = emulator.network_tx_pending?.() ?? 0;
  if (pending <= 0) {
    return 0;
  }
  let sent = 0;
  while (sent < pending) {
    const frame = emulator.network_tx_frame();
    if (!frame || frame.length === 0) {
      if (sent > 0) {
        markNetworkActivity(now);
      }
      return sent;
    }
    socket.send(frame);
    sent += 1;
  }
  markNetworkActivity(now);
  return sent;
}

function shouldPollNetworkTx(now) {
  if (now - state.lastNetworkActivityAt < NETWORK_IDLE_FAST_MS) {
    return true;
  }
  return (
    state.lastNetworkTxPollAt === 0 ||
    now - state.lastNetworkTxPollAt >= NETWORK_TX_POLL_INTERVAL_MS
  );
}

function connect() {
  clearRetry();
  setStatus("connecting");
  socket = new WebSocket(networkUrl());
  socket.binaryType = "arraybuffer";
  socket.onopen = () => setStatus("connected");
  socket.onclose = () => {
    socket = undefined;
    setStatus("offline");
    retryTimer = setTimeout(connect, RETRY_MS);
  };
  socket.onerror = () => setStatus("offline");
  socket.onmessage = (event) =>
    withEmulatorAccess(() => injectFrame(event.data)).catch((error) => {
      postMessage({ error: error?.message ?? String(error), event: "error" });
    });
}

function injectFrame(data) {
  const emulator = state.emulator;
  if (!emulator) {
    return;
  }
  const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : data;
  if (bytes?.length) {
    markNetworkActivity();
    emulator.inject_network_frame(bytes);
  }
}

function networkUrl() {
  const url = new URL(self.location.href);
  url.pathname = "/webboxvm-net";
  url.search = "";
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}

function clearRetry() {
  if (retryTimer) {
    clearTimeout(retryTimer);
    retryTimer = undefined;
  }
}

function setStatus(status) {
  if (state.networkStatus === status) {
    return;
  }
  state.networkStatus = status;
  if (status === "connected") {
    markNetworkActivity();
  }
  postMessage({ event: "network", status });
}

function markNetworkActivity(now = performance.now()) {
  state.lastNetworkActivityAt = now;
}
