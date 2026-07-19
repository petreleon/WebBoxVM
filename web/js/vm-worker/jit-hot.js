import { compileJitBlockEntry, jitBlockKey } from "./jit-compile.js?v=20260720-firmware-fast-boot-r2";
import { pcForCore, prepareNextJitCore } from "./jit-core.js?v=20260720-firmware-fast-boot-r2";
import { recordJitFallback, recordJitReject, recordJitSkip } from "./jit-stats.js?v=20260720-firmware-fast-boot-r2";
import { tryRunCachedJitBlock } from "./jit-run.js?v=20260720-firmware-fast-boot-r2";
import { JIT_HOT_THRESHOLD, JIT_MAX_HIT_SITES, state } from "./state.js?v=20260720-firmware-fast-boot-r2";

export function tryRunOrCompileNextJitBlock(emulator = state.emulator) {
  if (!state.jitEnabled || !emulator) {
    return false;
  }
  const coreId = prepareNextJitCore(emulator);
  if (!Number.isInteger(coreId) || coreId < 0) {
    return false;
  }
  return tryRunOrCompileJitBlock(coreId, emulator);
}

export function tryRunOrCompileJitBlock(coreId = 0, emulator = state.emulator) {
  if (!state.jitEnabled || !emulator) {
    return false;
  }

  const pc = pcForCore(emulator, coreId);
  const key = jitBlockKey(coreId, pc);
  const cached = state.jitBlocks.get(key);
  if (cached) {
    const result = tryRunCachedJitBlock(coreId, key, cached, pc, emulator);
    if (result === true) {
      return true;
    }
    if (result.invalidated) {
      state.jitBlocks.delete(key);
      if (result.rejected) {
        state.jitRejectedBlocks.add(key);
        recordJitReject(key, pc, result.error, coreId, emulator);
      }
    } else {
      recordJitFallback(key, pc, result.error, coreId, emulator);
      return false;
    }
  }

  if (state.jitRejectedBlocks.has(key) || state.jitSkippedBlocks.has(key)) {
    return false;
  }

  const hits = (state.jitBlockHits.get(key) ?? 0) + 1;
  if (hits < JIT_HOT_THRESHOLD) {
    state.jitBlockHits.set(key, hits);
    evictOldestWarmupHitIfNeeded();
    return false;
  }
  state.jitBlockHits.delete(key);

  return compileAndRunJitBlock({ coreId, key, pc, emulator });
}

async function compileAndRunJitBlock({ coreId, key, pc, emulator }) {
  if (state.emulator !== emulator) {
    return false;
  }
  const entry = await compileJitBlockEntry(coreId, pc, emulator);
  if (!state.running || state.emulator !== emulator || entry.compiled === false) {
    if (entry.compiled === false) {
      if (entry.skipped) {
        state.jitSkippedBlocks.add(key);
        recordJitSkip(key, pc, entry.error, coreId, emulator);
      } else {
        state.jitRejectedBlocks.add(key);
        recordJitReject(key, pc, entry.error, coreId, emulator);
      }
    }
    return false;
  }

  if (!entry) {
    return false;
  }
  return tryRunCachedJitBlock(coreId, key, entry, pc, emulator) === true;
}

function evictOldestWarmupHitIfNeeded() {
  if (state.jitBlockHits.size <= JIT_MAX_HIT_SITES) {
    return;
  }
  state.jitBlockHits.delete(state.jitBlockHits.keys().next().value);
}
