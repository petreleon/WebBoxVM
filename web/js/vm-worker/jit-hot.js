import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { recordJitFallback, recordJitReject, recordJitSkip } from "./jit-stats.js";
import { runCachedJitBlock } from "./jit-run.js";
import { JIT_HOT_THRESHOLD, state } from "./state.js";

export function tryRunOrCompileJitBlock(coreId = 0, emulator = state.emulator) {
  if (!state.jitEnabled || !emulator) {
    return false;
  }

  const pc = emulator.pc();
  const key = jitBlockKey(coreId, pc);
  const cached = state.jitBlocks.get(key);
  if (cached) {
    const result = runCachedJitBlock(coreId, key, cached, pc, emulator);
    if (result.committed) {
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
  state.jitBlockHits.set(key, hits);
  if (hits < JIT_HOT_THRESHOLD) {
    return false;
  }

  return compileAndRunJitBlock({ coreId, key, pc, emulator });
}

async function compileAndRunJitBlock({ coreId, key, pc, emulator }) {
  if (state.emulator !== emulator) {
    return false;
  }
  const compiled = await compileJitBlock({ coreId, pc, emulator });
  if (!state.running || state.emulator !== emulator || !compiled.compiled) {
    if (!compiled.compiled) {
      if (compiled.skipped) {
        state.jitSkippedBlocks.add(key);
        recordJitSkip(key, pc, compiled.error, coreId, emulator);
      } else {
        state.jitRejectedBlocks.add(key);
        recordJitReject(key, pc, compiled.error, coreId, emulator);
      }
    }
    return false;
  }

  const entry = state.jitBlocks.get(key);
  if (!entry) {
    return false;
  }
  return runCachedJitBlock(coreId, key, entry, pc, emulator).committed;
}
