import {
  UART_FLUSH_BYTES,
  UART_FLUSH_INTERVAL_MS,
  UART_POLL_INTERVAL_MS,
  state,
} from "./state.js?v=20260904-virgl-depth-vertex-color-r1";
import { isInputResponsive } from "./uart-input.js?v=20260904-virgl-depth-vertex-color-r1";

export function drainUart(now, emulator = state.emulator) {
  const responsive = isInputResponsive(now);
  if (!emulator || !shouldPollUart(now, state.lastUartPollAt, responsive)) {
    return false;
  }
  state.lastUartPollAt = now;
  const uartLen = emulator.uart_output_len();
  const pendingBytes = uartLen - state.lastUart;
  if (!shouldFlushUart(pendingBytes, now, state.lastUartFlushAt, responsive)) {
    return false;
  }
  const output = emulator.uart_output_since(state.lastUart);
  state.lastUart = uartLen;
  state.lastUartFlushAt = now;
  postMessage({ event: "uart", output });
  return true;
}

export function shouldFlushUart(pendingBytes, now, lastFlushAt, responsive = false) {
  if (pendingBytes <= 0) {
    return false;
  }
  return (
    responsive ||
    pendingBytes >= UART_FLUSH_BYTES ||
    now - lastFlushAt >= UART_FLUSH_INTERVAL_MS
  );
}

export function shouldPollUart(now, lastPollAt, responsive = false) {
  return responsive || lastPollAt === 0 || now - lastPollAt >= UART_POLL_INTERVAL_MS;
}
