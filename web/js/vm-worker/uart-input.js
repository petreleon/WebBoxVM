import {
  COOPERATIVE_STEP_SLICE,
  INTERACTIVE_STEP_SLICE,
  INTERACTIVE_WINDOW_MS,
  state,
} from "./state.js?v=20260904-virgl-mixed-depth-batch-r1";

const INPUT_TYPES = new Set(["sendUartBytes", "sendUartInput"]);

export function beginUrgentUartMessage({ payload = {}, type }) {
  if (!INPUT_TYPES.has(type)) {
    return false;
  }
  if (inputLength(payload.input) === 0) {
    return false;
  }
  state.urgentUartWaiters += 1;
  state.vcpuPool?.interrupt?.();
  return true;
}

export function finishUrgentUartMessage(urgent) {
  if (!urgent) {
    return;
  }
  state.urgentUartWaiters = Math.max(0, state.urgentUartWaiters - 1);
}

export function injectUartMessage(
  type,
  input,
  emulator = state.emulator,
  now = performance.now(),
) {
  if (!INPUT_TYPES.has(type) || !emulator || inputLength(input) === 0) {
    return false;
  }
  if (type === "sendUartBytes") {
    emulator.send_uart_bytes(input);
  } else {
    emulator.send_uart_input(input);
  }
  state.lastUartInputAt = now;
  state.uartNeedsGuestService = true;
  return true;
}

export function markUartGuestServiced(now = performance.now()) {
  if (!state.uartNeedsGuestService) {
    return false;
  }
  state.uartNeedsGuestService = false;
  state.lastUartInputAt = now;
  return true;
}

export function responsiveStepSlice(now = performance.now()) {
  const steadyState =
    state.executionMode === "parallel-wasm"
      ? state.stepSlice
      : Math.min(state.stepSlice, COOPERATIVE_STEP_SLICE);
  return isInputResponsive(now)
    ? Math.min(steadyState, INTERACTIVE_STEP_SLICE)
    : steadyState;
}

export function isInputResponsive(now = performance.now()) {
  return (
    state.urgentUartWaiters > 0 ||
    state.uartNeedsGuestService ||
    now - state.lastUartInputAt < INTERACTIVE_WINDOW_MS
  );
}

export function resetUartInput() {
  state.lastUartInputAt = Number.NEGATIVE_INFINITY;
  state.uartNeedsGuestService = false;
}

function inputLength(input) {
  return typeof input === "string" || ArrayBuffer.isView(input) ? input.length : 0;
}
