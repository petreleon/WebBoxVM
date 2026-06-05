import { state } from "./state.js?v=20260606-jitprobe";

const MAX_REJECT_LOG = 16;

export function jitStats() {
  return {
    cacheBlocks: state.jitBlocks.size,
    enabled: state.jitEnabled,
    hitSites: state.jitBlockHits.size,
    recentRejects: state.jitRejectLog.slice(-MAX_REJECT_LOG),
    rejectedBlocks: state.jitRejectedBlocks.size,
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
