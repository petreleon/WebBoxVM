import { requireEmulator } from "./lifecycle.js";
import { JIT_MAX_BLOCKS, state } from "./state.js";

export async function compileJitBlock({ coreId = 0 } = {}) {
  requireEmulator();
  const owner = state.emulator;
  if (!state.wasmExports?.memory) {
    throw new Error("Wasm memory export is unavailable for JIT blocks");
  }

  const pc = owner.pc();
  const bytes = owner.jit_compile_current_block(coreId);
  if (!bytes.length) {
    return {
      compiled: false,
      error: owner.jit_last_error(),
      pc,
    };
  }

  const key = jitBlockKey(coreId, pc);
  const steps = owner.jit_last_block_steps();
  const startPc = owner.jit_last_block_start_pc();
  const startPa = owner.jit_last_block_start_pa();
  const exitPc = owner.jit_last_block_exit_pc();
  const alternateExitPc = owner.jit_last_block_alternate_exit_pc();
  const dynamicExit = owner.jit_last_block_dynamic_exit();
  const rawHash = owner.jit_last_block_raw_hash();
  const { instance, module } = await WebAssembly.instantiate(bytes, {
    env: { memory: state.wasmExports.memory },
  });
  if (state.emulator !== owner) {
    return {
      compiled: false,
      error: "VM changed while compiling JIT block",
      pc,
    };
  }
  evictOldestJitBlockIfNeeded();
  state.jitBlocks.set(key, {
    alternateExitPc,
    dynamicExit,
    exitPc,
    instance,
    module,
    rawHash,
    startPa,
    startPc,
    steps,
  });
  state.jitRejectedBlocks.delete(key);

  return {
    compiled: true,
    bytes: bytes.length,
    alternateExitPc,
    dynamicExit,
    exitPc,
    pc,
    rawHash,
    startPa,
    statePtr: owner.jit_state_ptr(),
    stateSize: owner.jit_state_size(),
    steps,
  };
}

export function jitBlockKey(coreId, pc) {
  return `${coreId}:${pc.toString(16)}`;
}

function evictOldestJitBlockIfNeeded() {
  if (state.jitBlocks.size < JIT_MAX_BLOCKS) {
    return;
  }
  const evictedKey = state.jitBlocks.keys().next().value;
  state.jitBlocks.delete(evictedKey);
  state.jitBlockHits.delete(evictedKey);
  state.jitRejectedBlocks.delete(evictedKey);
}
