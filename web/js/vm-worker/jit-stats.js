import { state } from "./state.js?v=20260903-virgl-blend-r1";

const MAX_REJECT_LOG = 16;

export function jitStats() {
  state.jitStatsFingerprint = jitStatsFingerprint();
  return buildJitStats();
}

export function changedJitStats() {
  const fingerprint = jitStatsFingerprint();
  if (fingerprint === state.jitStatsFingerprint) {
    return undefined;
  }
  state.jitStatsFingerprint = fingerprint;
  return buildJitStats();
}

function buildJitStats() {
  return {
    cacheBlocks: state.jitBlocks.size,
    enabled: state.jitEnabled,
    fallbackCount: state.jitFallbackCount,
    hitSites: state.jitBlockHits.size,
    lastFallback: state.jitLastFallback,
    recentRejects: state.jitRejectLog,
    recentSkips: state.jitSkipLog,
    rejectedBlocks: state.jitRejectedBlocks.size,
    skippedBlocks: state.jitSkippedBlocks.size,
  };
}

export function recordJitSkip(key, pc, error, coreId = 0, emulator = state.emulator) {
  state.jitStatsVersion += 1;
  state.jitSkipLog.push(jitEvent(key, pc, error, "unknown JIT skip", coreId, emulator));
  if (state.jitSkipLog.length > MAX_REJECT_LOG) {
    state.jitSkipLog.splice(0, state.jitSkipLog.length - MAX_REJECT_LOG);
  }
}

export function recordJitFallback(key, pc, error, coreId = 0, emulator = state.emulator) {
  state.jitStatsVersion += 1;
  state.jitFallbackCount += 1;
  const errorText = error || "unknown JIT fallback";
  const keyText = jitEventKey(key, coreId);
  const pcHex = pc.toString(16);
  if (isSameFallbackEvent(state.jitLastFallback, keyText, pcHex, errorText)) {
    return;
  }
  state.jitLastFallback = jitEvent(key, pc, errorText, "unknown JIT fallback", coreId, emulator);
}

export function recordJitReject(key, pc, error, coreId = 0, emulator = state.emulator) {
  state.jitStatsVersion += 1;
  state.jitRejectLog.push(jitEvent(key, pc, error, "unknown JIT rejection", coreId, emulator));
  if (state.jitRejectLog.length > MAX_REJECT_LOG) {
    state.jitRejectLog.splice(0, state.jitRejectLog.length - MAX_REJECT_LOG);
  }
}

export function currentJitInstruction(coreId = 0, emulator = state.emulator) {
  const snapshot = emulator?.current_instruction?.(coreId);
  if (typeof snapshot !== "string" || snapshot.length === 0) {
    return undefined;
  }
  try {
    return JSON.parse(snapshot);
  } catch {
    return { text: snapshot };
  }
}

function jitEvent(key, pc, error, fallbackError, coreId, emulator) {
  const event = {
    error: error || fallbackError,
    key: jitEventKey(key, coreId),
    pc: pc.toString(16),
  };
  const instruction = currentJitInstruction(coreId, emulator);
  if (instruction) {
    event.instruction = instruction;
  }
  return event;
}

function jitEventKey(key, coreId) {
  if (typeof key === "bigint") {
    return `${coreId}:${key.toString(16)}`;
  }
  return key;
}

function isSameFallbackEvent(event, key, pc, error) {
  return event?.key === key && event.pc === pc && event.error === error;
}

function jitStatsFingerprint() {
  return `${state.jitEnabled ? 1 : 0}:${state.jitBlocks.size}:${state.jitBlockHits.size}:${
    state.jitFallbackCount
  }:${state.jitRejectedBlocks.size}:${state.jitSkippedBlocks.size}:${state.jitRejectLog.length}:${
    state.jitSkipLog.length
  }:${state.jitStatsVersion}`;
}
