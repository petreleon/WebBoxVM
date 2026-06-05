import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { recordJitReject } from "./jit-stats.js";
import { runCachedJitBlock } from "./jit-run.js";
import { JIT_HOT_THRESHOLD, state } from "./state.js";

export async function tryRunOrCompileJitBlock(coreId = 0) {
  if (!state.jitEnabled || !state.emulator) {
    return false;
  }

  const pc = state.emulator.pc();
  const key = jitBlockKey(coreId, pc);
  const cached = state.jitBlocks.get(key);
  if (cached) {
    const result = runCachedJitBlock(coreId, key, cached);
    if (result.committed) {
      return true;
    }
    if (result.invalidated) {
      state.jitBlocks.delete(key);
    } else {
      return false;
    }
  }

  if (state.jitRejectedBlocks.has(key)) {
    return false;
  }

  const hits = (state.jitBlockHits.get(key) ?? 0) + 1;
  state.jitBlockHits.set(key, hits);
  if (hits < JIT_HOT_THRESHOLD) {
    return false;
  }

  const compiled = await compileJitBlock({ coreId });
  if (!state.running || !compiled.compiled) {
    if (!compiled.compiled) {
      state.jitRejectedBlocks.add(key);
      recordJitReject(key, pc, compiled.error);
    }
    return false;
  }

  const entry = state.jitBlocks.get(key);
  if (!entry) {
    return false;
  }
  return runCachedJitBlock(coreId, key, entry).committed;
}
