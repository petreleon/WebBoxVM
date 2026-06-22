import { state } from "./state.js";

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

export function drainNetworkTx() {
  if (!state.emulator || !socket || socket.readyState !== WebSocket.OPEN) {
    return 0;
  }
  const pending = state.emulator.network_tx_pending?.() ?? 0;
  if (pending <= 0) {
    return 0;
  }
  let sent = 0;
  while (sent < pending) {
    const frame = state.emulator.network_tx_frame();
    if (!frame || frame.length === 0) {
      if (sent > 0) {
        markNetworkActivity();
      }
      return sent;
    }
    socket.send(frame);
    sent += 1;
  }
  markNetworkActivity();
  return sent;
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
  socket.onmessage = (event) => injectFrame(event.data);
}

function injectFrame(data) {
  if (!state.emulator) {
    return;
  }
  const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : data;
  if (bytes?.length) {
    markNetworkActivity();
    state.emulator.inject_network_frame(bytes);
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

function markNetworkActivity() {
  state.lastNetworkActivityAt = performance.now();
}
