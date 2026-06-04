import { metrics } from "./lifecycle.js";
import { AUTOSAVE_INTERVAL_MS, METRICS_INTERVAL_MS, state } from "./state.js";

export function maybePostMetrics() {
  const now = performance.now();
  if (now - state.lastMetricsAt < METRICS_INTERVAL_MS) {
    return;
  }
  state.lastMetricsAt = now;
  postMetrics();
}

export function postMetrics({ force = false } = {}) {
  if (!state.emulator) {
    return;
  }
  if (force) {
    state.lastMetricsAt = performance.now();
  }
  postMessage({ event: "metrics", metrics: metrics() });
}

export function maybeRequestAutosave() {
  const generation = state.emulator.install_disk_generation();
  if (generation === state.lastAutosaveGeneration) {
    return;
  }

  const now = performance.now();
  if (now - state.lastAutosaveAt < AUTOSAVE_INTERVAL_MS) {
    return;
  }

  state.lastAutosaveAt = now;
  state.lastAutosaveGeneration = generation;
  postMessage({ event: "autosave" });
}
