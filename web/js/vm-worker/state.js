export const AUTOSAVE_INTERVAL_MS = 60_000;
export const AUTOSAVE_POLL_MS = 1000;
export const DEFAULT_STEP_SLICE = 5_000_000;
export const MAX_STEP_SLICE = 50_000_000;
export const NETWORK_STEP_SLICE = 1_000_000;
export const NETWORK_IDLE_FAST_MS = 1500;
export const NETWORK_TX_POLL_INTERVAL_MS = 16;
export const UART_FLUSH_BYTES = 8192;
export const UART_FLUSH_INTERVAL_MS = 50;
export const UART_POLL_INTERVAL_MS = 16;
export const INTERACTIVE_STEP_SLICE = 100_000;
export const INTERACTIVE_WINDOW_MS = 750;
export const COOPERATIVE_STEP_SLICE = 500_000;
export const MAX_FRAME_MS = 32;
export const MAX_FRAME_BATCHES = 128;
export const METRICS_INTERVAL_MS = 250;
export const GPU_SCANOUT_POLL_INTERVAL_MS = 1000 / 60;
export const GPU_3D_POLL_INTERVAL_MS = 1000 / 60;
export const DEFAULT_JIT_ENABLED = false;
export const JIT_HOT_THRESHOLD = 2;
export const JIT_MAX_HIT_SITES = 1024;
export const JIT_MAX_BLOCKS = 4096;
export const JIT_PROBE_STEP_SLICE = COOPERATIVE_STEP_SLICE;

export const state = {
  emulator: undefined,
  executionMode: "cooperative",
  jitEnabled: DEFAULT_JIT_ENABLED,
  jitBlockHits: new Map(),
  jitBlocks: new Map(),
  jitImports: undefined,
  jitFallbackCount: 0,
  jitLastFallback: undefined,
  jitRejectLog: [],
  jitRejectedBlocks: new Set(),
  jitSkipLog: [],
  jitSkippedBlocks: new Set(),
  jitStatsFingerprint: undefined,
  jitStatsVersion: 0,
  jitStatePtr: undefined,
  jitStateSize: undefined,
  gpuResetGeneration: undefined,
  lastAutosaveAt: 0,
  lastAutosaveGeneration: 0n,
  lastAutosavePollAt: 0,
  lastGpuScanoutPollAt: Number.NEGATIVE_INFINITY,
  lastGpu3dPollAt: Number.NEGATIVE_INFINITY,
  lastMetricsAt: 0,
  lastNetworkActivityAt: 0,
  lastNetworkTxPollAt: 0,
  lastUart: 0,
  lastUartFlushAt: 0,
  lastUartInputAt: Number.NEGATIVE_INFINITY,
  lastUartPollAt: 0,
  networkStatus: "offline",
  numCores: 0,
  parallelTransitionDeferred: false,
  pumpScheduled: false,
  running: false,
  stepSlice: DEFAULT_STEP_SLICE,
  threadedWasm: undefined,
  uartNeedsGuestService: false,
  urgentUartWaiters: 0,
  vcpuPool: undefined,
  wasmExports: undefined,
  wasmFallbackReason: undefined,
  wasmReady: false,
};

export function resetJitState() {
  state.jitBlocks = new Map();
  state.jitImports = undefined;
  state.jitFallbackCount = 0;
  state.jitLastFallback = undefined;
  state.jitBlockHits = new Map();
  state.jitRejectLog = [];
  state.jitRejectedBlocks = new Set();
  state.jitSkipLog = [];
  state.jitSkippedBlocks = new Set();
  state.jitStatsFingerprint = undefined;
  state.jitStatsVersion = 0;
  state.jitStatePtr = undefined;
  state.jitStateSize = undefined;
}
