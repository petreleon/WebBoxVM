import { state } from "./state.js";

const MAX_REJECT_LOG = 16;

export function jitStats() {
  return {
    cacheBlocks: state.jitBlocks.size,
    enabled: state.jitEnabled,
    fallbackCount: state.jitFallbackCount,
    hitSites: state.jitBlockHits.size,
    lastFallback: state.jitLastFallback,
    recentRejects: state.jitRejectLog.slice(-MAX_REJECT_LOG),
    rejectedBlocks: state.jitRejectedBlocks.size,
  };
}

export function recordJitFallback(key, pc, error) {
  state.jitFallbackCount += 1;
  state.jitLastFallback = {
    error: error || "unknown JIT fallback",
    key,
    pc: pc.toString(16),
  };
}

export function recordJitReject(key, pc, error) {
  state.jitRejectLog.push({
    error: error || "unknown JIT rejection",
    key,
    pc: pc.toString(16),
  });
  if (state.jitRejectLog.length > MAX_REJECT_LOG) {
    state.jitRejectLog.splice(0, state.jitRejectLog.length - MAX_REJECT_LOG);
  }
}
