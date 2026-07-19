import { metrics } from "./lifecycle.js?v=20260720-firmware-fast-boot-r2";
import { AUTOSAVE_INTERVAL_MS, AUTOSAVE_POLL_MS, METRICS_INTERVAL_MS, state } from "./state.js?v=20260720-firmware-fast-boot-r2";

export function maybePostMetrics(now = performance.now(), emulator = state.emulator) {
  if (now - state.lastMetricsAt < METRICS_INTERVAL_MS) {
    return;
  }
  state.lastMetricsAt = now;
  postMetrics({ emulator });
}

export function postMetrics({ force = false, now, emulator = state.emulator } = {}) {
  if (!emulator) {
    return;
  }
  if (force) {
    state.lastMetricsAt = now ?? performance.now();
  }
  postMessage({
    event: "metrics",
    metrics: metrics({ emulator, includeUnchangedJitStats: force }),
  });
}

export function maybeRequestAutosave(now = performance.now(), emulator = state.emulator) {
  if (now - state.lastAutosavePollAt < AUTOSAVE_POLL_MS) {
    return;
  }
  state.lastAutosavePollAt = now;

  const generation = emulator.install_disk_generation();
  if (generation === state.lastAutosaveGeneration) {
    return;
  }

  if (now - state.lastAutosaveAt < AUTOSAVE_INTERVAL_MS) {
    return;
  }

  state.lastAutosaveAt = now;
  state.lastAutosaveGeneration = generation;
  postMessage({ event: "autosave", installDiskGeneration: generation });
}
