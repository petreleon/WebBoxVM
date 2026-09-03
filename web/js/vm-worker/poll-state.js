import { state } from "./state.js?v=20260903-webgpu-virtio-r4";
import { resetUartInput } from "./uart-input.js?v=20260903-webgpu-virtio-r4";

export function resetVmPollState(now = performance.now()) {
  resetUartInput();
  state.gpuResetGeneration = undefined;
  state.lastUart = 0;
  state.lastUartFlushAt = 0;
  state.lastUartPollAt = 0;
  state.lastNetworkTxPollAt = 0;
  state.lastGpuScanoutPollAt = Number.NEGATIVE_INFINITY;
  state.lastGpu3dPollAt = Number.NEGATIVE_INFINITY;
  state.lastMetricsAt = 0;
  state.lastAutosaveAt = now;
  state.lastAutosavePollAt = now;
}
