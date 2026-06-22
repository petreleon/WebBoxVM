import { compileJitBlock, jitBlockKey } from "./jit-compile.js";
import { postMetrics } from "./metrics-events.js";
import { state } from "./state.js";
import { requireEmulator } from "./lifecycle.js";

const ANY_DYNAMIC_EXIT_PC = 0xffffffffffffffffn;

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
    !state.emulator.jit_prepare_cached_block(
      coreId,
      entry.startPc,
      entry.startPa,
      entry.rawHash,
      entry.startPageGeneration,
      entry.endPageGeneration,
      entry.steps,
    )
  ) {
    const error = state.emulator.jit_last_error();
    const result = { committed: false, error, pc };
    if (!isCommitBoundaryError(error)) {
      result.invalidated = true;
    }
    return result;
  }

  const exitPc = entry.instance.exports.run(entry.statePtr);
  if (state.emulator.jit_helper_failed()) {
    return {
      committed: false,
      error: state.emulator.jit_last_error(),
      invalidated: true,
      pc,
      rejected: true,
    };
  }
  if (!isAllowedExit(exitPc, entry)) {
    state.jitBlocks.delete(key);
    return {
      committed: false,
      error: exitMismatchMessage(exitPc, entry),
      invalidated: true,
      pc,
    };
  }

  const committed = state.emulator.jit_commit_state_to_core(coreId, entry.steps, exitPc);
  return {
    committed,
    error: committed ? "" : state.emulator.jit_last_error(),
    exitPc,
    pc,
    steps: entry.steps,
  };
}

function canCommitNow(coreId, steps) {
  if (!state.emulator.jit_can_commit_block_now) {
    return "";
  }
  if (state.emulator.jit_can_commit_block_now(coreId, steps)) {
    return "";
  }
  return state.emulator.jit_last_error();
}

function isAllowedExit(exitPc, entry) {
  if (exitPc === entry.exitPc) {
    return true;
  }
  if (entry.dynamicExit && entry.alternateExitPc === ANY_DYNAMIC_EXIT_PC) {
    return true;
  }
  return entry.dynamicExit && exitPc === entry.alternateExitPc;
}

function exitMismatchMessage(exitPc, entry) {
  const actual = `0x${exitPc.toString(16)}`;
  const expected = `0x${entry.exitPc.toString(16)}`;
  if (!entry.dynamicExit) {
    return `JIT block returned ${actual} instead of ${expected}`;
  }
  if (entry.alternateExitPc === ANY_DYNAMIC_EXIT_PC) {
    return `JIT block returned ${actual} outside arbitrary dynamic exit`;
  }
  return `JIT block returned ${actual} outside ${expected}/0x${entry.alternateExitPc.toString(16)}`;
}

function isCommitBoundaryError(error) {
  return (
    error.startsWith("JIT block crosses ") ||
    error.startsWith("JIT commit is currently restricted") ||
    error.startsWith("JIT core mismatch") ||
    error === "cannot commit an empty JIT block"
  );
}
