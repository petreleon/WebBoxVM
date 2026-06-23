import { requireEmulator } from "./lifecycle.js";
import { JIT_MAX_BLOCKS, state } from "./state.js";

export async function compileJitBlock({ coreId = 0, pc: knownPc, emulator: owner } = {}) {
  owner ||= requireEmulator();
  if (!state.wasmExports?.memory) {
    throw new Error("Wasm memory export is unavailable for JIT blocks");
  }

  const pc = knownPc ?? owner.pc();
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
  const skipReason = compiledJitBlockSkipReason({
    blockEl: owner.jit_last_block_el(),
    usesGuestHelpers: owner.jit_last_block_uses_guest_helpers(),
  });
  if (skipReason) {
    return {
      compiled: false,
      error: skipReason,
      pc,
      skipped: true,
    };
  }
  const rawHash = owner.jit_last_block_raw_hash();
  const startPageGeneration = owner.jit_last_block_start_page_generation();
  const endPageGeneration = owner.jit_last_block_end_page_generation();
  const statePtr = owner.jit_state_ptr();
  const { instance, module } = await WebAssembly.instantiate(bytes, {
    env: {
      memory: state.wasmExports.memory,
      jitLoadGuest: (va, size) => owner.jit_load_guest(coreId, va, size),
      jitStoreGuest: (va, size, value) => owner.jit_store_guest(coreId, va, size, value),
      jitStorePairGuest: (va, size, value1, value2) =>
        owner.jit_store_pair_guest(coreId, va, size, value1, value2),
      jitLoadPairGuest: (va, size) => owner.jit_load_pair_guest(coreId, va, size),
      jitStoreQuadGuest: (va, size, value1, value2, value3, value4) =>
        owner.jit_store_quad_guest(coreId, va, size, value1, value2, value3, value4),
      jitLoadQuadGuest: (va, size) => owner.jit_load_quad_guest(coreId, va, size),
      jitReadSysReg: (sysregId) => owner.jit_read_sysreg(coreId, sysregId),
      jitStoreExclusivePair: (va, size, value1, value2) =>
        owner.jit_store_exclusive_pair(coreId, va, size, value1, value2),
      jitLoadExclusive: (va, size) => owner.jit_load_exclusive(coreId, va, size),
      jitStoreExclusive: (va, size, value) =>
        owner.jit_store_exclusive(coreId, va, size, value),
    },
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
    startPageGeneration,
    endPageGeneration,
    startPa,
    startPc,
    statePtr,
    steps,
  });
  state.jitRejectedBlocks.delete(key);
  state.jitSkippedBlocks.delete(key);

  return {
    compiled: true,
    bytes: bytes.length,
    alternateExitPc,
    dynamicExit,
    exitPc,
    pc,
    rawHash,
    startPageGeneration,
    endPageGeneration,
    startPa,
    statePtr,
    stateSize: owner.jit_state_size(),
    steps,
  };
}

export function jitBlockKey(coreId, pc) {
  if (coreId === 0) {
    return pc;
  }
  return `${coreId}:${pc.toString(16)}`;
}

export function compiledJitBlockSkipReason(_metadata) {
  return undefined;
}

function evictOldestJitBlockIfNeeded() {
  if (state.jitBlocks.size < JIT_MAX_BLOCKS) {
    return;
  }
  const evictedKey = state.jitBlocks.keys().next().value;
  state.jitBlocks.delete(evictedKey);
  state.jitBlockHits.delete(evictedKey);
  state.jitRejectedBlocks.delete(evictedKey);
  state.jitSkippedBlocks.delete(evictedKey);
}
