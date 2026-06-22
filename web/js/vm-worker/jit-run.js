import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { postMetrics } from "./metrics-events.js";
import { state } from "./state.js";
import { requireEmulator } from "./lifecycle.js";

const JIT_FINISH_COMMITTED = 0;
const JIT_FINISH_HELPER_REJECTED = 1;
const JIT_FINISH_EXIT_REJECTED = 3;

export async function runJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const emulator = state.emulator;
  const pc = emulator.pc();
  const key = jitBlockKey(coreId, pc);
  let entry = state.jitBlocks.get(key);

  if (!entry) {
    const compiled = await compileJitBlock({ coreId, pc });
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
  const knownPc = pc ?? emulator.pc();
  if (
    !emulator.jit_prepare_cached_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
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

  const exitPc = entry.instance.exports.run(entry.statePtr);
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
