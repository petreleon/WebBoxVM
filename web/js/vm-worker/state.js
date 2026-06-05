export const AUTOSAVE_INTERVAL_MS = 5000;
export const DEFAULT_STEP_SLICE = 1_000_000;
export const MAX_FRAME_MS = 24;
export const MAX_FRAME_BATCHES = 8;
export const METRICS_INTERVAL_MS = 100;
export const DEFAULT_JIT_ENABLED = true;
export const JIT_HOT_THRESHOLD = 2;
export const JIT_MAX_BLOCKS = 4096;

export const state = {
  emulator: undefined,
  jitEnabled: DEFAULT_JIT_ENABLED,
  jitBlockHits: new Map(),
  jitBlocks: new Map(),
  jitRejectedBlocks: new Set(),
  lastAutosaveAt: 0,
  lastAutosaveGeneration: 0n,
  lastMetricsAt: 0,
  lastUart: 0,
  pumpScheduled: false,
  running: false,
  stepSlice: DEFAULT_STEP_SLICE,
  wasmExports: undefined,
  wasmReady: false,
};

export function resetJitState() {
  state.jitBlocks = new Map();
  state.jitBlockHits = new Map();
  state.jitRejectedBlocks = new Set();
}
