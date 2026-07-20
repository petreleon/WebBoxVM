import { compileJitBlock, jitBlockKey } from "./jit-compile.js?v=20260720-input-latency-r4";
import { pcForCore } from "./jit-core.js?v=20260720-input-latency-r4";
import { postMetrics } from "./metrics-events.js?v=20260720-input-latency-r4";
import { state } from "./state.js?v=20260720-input-latency-r4";
import { requireEmulator } from "./lifecycle.js?v=20260720-input-latency-r4";

const JIT_FINISH_COMMITTED = 0;
const JIT_FINISH_HELPER_REJECTED = 1;
const JIT_FINISH_EXIT_REJECTED = 3;

export async function runJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const emulator = state.emulator;
  const pc = pcForCore(emulator, coreId);
  const key = jitBlockKey(coreId, pc);
  let entry = state.jitBlocks.get(key);

  if (!entry) {
    const compiled = await compileJitBlock({ coreId, pc, emulator });
    if (!compiled.compiled) {
      return compiled;
    }
    entry = state.jitBlocks.get(key);
  }

  const result = runCachedJitBlock(coreId, key, entry, pc, emulator);
  if (result.committed) {
    postMetrics({ force: true });
  }
  return { compiled: true, ...result };
}

export function runCachedJitBlock(coreId, key, entry, pc, emulator = state.emulator) {
  return runCachedJitBlockCore(coreId, key, entry, pc, emulator, false);
}

export function tryRunCachedJitBlock(coreId, key, entry, pc, emulator = state.emulator) {
  return runCachedJitBlockCore(coreId, key, entry, pc, emulator, true);
}

function runCachedJitBlockCore(coreId, key, entry, pc, emulator, fastResult) {
  const knownPc = pc ?? pcForCore(emulator, coreId);
  if (
    !emulator.jit_prepare_cached_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
      entry.memoryGeneration,
      entry.startPageGeneration,
      entry.endPageGeneration,
      entry.steps,
    )
  ) {
    const error = emulator.jit_last_error();
    const result = { committed: false, error, pc: knownPc };
    if (!isCommitBoundaryError(error)) {
      result.invalidated = true;
    }
    return result;
  }

  const exitPc = entry.run(entry.statePtr);
  const finish = emulator.jit_finish_cached_block(
    coreId,
    entry.steps,
    exitPc,
    entry.exitPc,
    entry.alternateExitPc,
    entry.dynamicExit,
  );
  if (finish === JIT_FINISH_HELPER_REJECTED) {
    return {
      committed: false,
      error: emulator.jit_last_error(),
      invalidated: true,
      pc: knownPc,
      rejected: true,
    };
  }
  if (finish === JIT_FINISH_EXIT_REJECTED) {
    state.jitBlocks.delete(key);
    return {
      committed: false,
      error: emulator.jit_last_error(),
      invalidated: true,
      pc: knownPc,
    };
  }
  if (finish !== JIT_FINISH_COMMITTED) {
    return {
      committed: false,
      error: emulator.jit_last_error(),
      exitPc,
      pc: knownPc,
      steps: entry.steps,
    };
  }

  if (fastResult) {
    return true;
  }
  return {
    committed: true,
    error: "",
    exitPc,
    pc: knownPc,
    steps: entry.steps,
  };
}

function isCommitBoundaryError(error) {
  return (
    error.startsWith("JIT block crosses ") ||
    error.startsWith("JIT commit is currently restricted") ||
    error.startsWith("JIT core mismatch") ||
    error === "cannot commit an empty JIT block"
  );
}
