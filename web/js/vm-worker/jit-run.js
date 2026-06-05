import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { postMetrics } from "./metrics-events.js";
import { state } from "./state.js";
import { requireEmulator } from "./lifecycle.js";

export async function runJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const pc = state.emulator.pc();
  const key = jitBlockKey(coreId, pc);
  let entry = state.jitBlocks.get(key);

  if (!entry) {
    const compiled = await compileJitBlock({ coreId });
    if (!compiled.compiled) {
      return compiled;
    }
    entry = state.jitBlocks.get(key);
  }

  const result = runCachedJitBlock(coreId, key, entry);
  if (result.committed) {
    postMetrics({ force: true });
  }
  return { compiled: true, ...result };
}

export function runCachedJitBlock(coreId, key, entry) {
  const pc = state.emulator.pc();
  if (
    !state.emulator.jit_validate_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
      entry.steps,
    )
  ) {
    return {
      committed: false,
      error: state.emulator.jit_last_error(),
      invalidated: true,
      pc,
    };
  }

  if (!state.emulator.jit_sync_state_from_core(coreId)) {
    return { committed: false, error: state.emulator.jit_last_error(), pc };
  }

  const exitPc = entry.instance.exports.run(state.emulator.jit_state_ptr());
  if (exitPc !== entry.exitPc) {
    state.jitBlocks.delete(key);
    return {
      committed: false,
      error: `JIT block returned 0x${exitPc.toString(16)} instead of 0x${entry.exitPc.toString(16)}`,
      invalidated: true,
      pc,
    };
  }

  const committed = state.emulator.jit_commit_state_to_core(coreId, entry.steps, entry.exitPc);
  return {
    committed,
    error: committed ? "" : state.emulator.jit_last_error(),
    exitPc,
    pc,
    steps: entry.steps,
  };
}
