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
    return;
  }
  for (;;) {
    const frame = state.emulator.network_tx_frame();
    if (!frame || frame.length === 0) {
      return;
    }
    socket.send(frame);
  }
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
  postMessage({ event: "network", status });
}
