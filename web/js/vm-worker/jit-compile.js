import { requireEmulator } from "./lifecycle.js?v=20260904-virgl-depth-r1";
import { pcForCore } from "./jit-core.js?v=20260904-virgl-depth-r1";
import { JIT_MAX_BLOCKS, state } from "./state.js?v=20260904-virgl-depth-r1";

const MIN_CACHED_JIT_STEPS = 2;

export async function compileJitBlock({
  coreId = 0,
  pc: knownPc,
  emulator: owner,
  details = true,
} = {}) {
  owner ||= requireEmulator();
  return compileJitBlockFor(owner, coreId, knownPc, details);
}

export async function compileJitBlockEntry(coreId = 0, knownPc, owner) {
  owner ||= requireEmulator();
  return compileJitBlockFor(owner, coreId, knownPc, false);
}

async function compileJitBlockFor(owner, coreId, knownPc, details) {
  if (!state.wasmExports?.memory) {
    throw new Error("Wasm memory export is unavailable for JIT blocks");
  }

  const pc = knownPc ?? pcForCore(owner, coreId);
  const bytes = owner.jit_compile_current_block(coreId);
  if (!bytes.length) {
    return {
      compiled: false,
      error: owner.jit_last_error(),
      pc,
    };
  }

  const key = jitBlockKey(coreId, pc);
  const metadata = readJitBlockMetadata(owner);
  const steps = Number(metadata[0]);
  const startPc = metadata[1];
  const startPa = metadata[2];
  const exitPc = metadata[3];
  const alternateExitPc = metadata[4];
  const dynamicExit = metadata[5] !== 0n;
  const rawHash = metadata[6];
  const memoryGeneration = metadata[7];
  const startPageGeneration = metadata[8];
  const endPageGeneration = metadata[9];
  if (steps < MIN_CACHED_JIT_STEPS) {
    return {
      compiled: false,
      error: "single-instruction JIT block skipped",
      pc,
      skipped: true,
    };
  }
  const statePtr = jitStatePtr(owner);
  const { instance } = await WebAssembly.instantiate(bytes, jitImports(owner, coreId));
  if (state.emulator !== owner) {
    return {
      compiled: false,
      error: "VM changed while compiling JIT block",
      pc,
    };
  }
  evictOldestJitBlockIfNeeded();
  const entry = {
    alternateExitPc,
    dynamicExit,
    exitPc,
    rawHash,
    memoryGeneration,
    run: instance.exports.run,
    startPageGeneration,
    endPageGeneration,
    startPa,
    startPc,
    statePtr,
    steps,
  };
  state.jitBlocks.set(key, entry);
  state.jitRejectedBlocks.delete(key);
  state.jitSkippedBlocks.delete(key);

  if (!details) {
    return entry;
  }

  return {
    compiled: true,
    bytes: bytes.length,
    alternateExitPc,
    dynamicExit,
    exitPc,
    pc,
    rawHash,
    memoryGeneration,
    startPageGeneration,
    endPageGeneration,
    startPa,
    statePtr,
    stateSize: jitStateSize(owner),
    steps,
  };
}

function jitImports(owner, coreId) {
  const memory = state.wasmExports.memory;
  const cached = state.jitImports;
  if (cached?.owner === owner && cached.coreId === coreId && cached.memory === memory) {
    return cached.imports;
  }
  const imports = {
    env: {
      memory,
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
      jitLoadExclusivePair: (va, size) => owner.jit_load_exclusive_pair(coreId, va, size),
      jitStoreExclusive: (va, size, value) =>
        owner.jit_store_exclusive(coreId, va, size, value),
    },
  };
  state.jitImports = { coreId, imports, memory, owner };
  return imports;
}

export function jitBlockKey(coreId, pc) {
  if (coreId === 0) {
    return pc;
  }
  return `${coreId}:${pc.toString(16)}`;
}

function readJitBlockMetadata(owner) {
  return owner.jit_last_block_metadata();
}

function jitStatePtr(owner) {
  state.jitStatePtr ??= owner.jit_state_ptr();
  return state.jitStatePtr;
}

function jitStateSize(owner) {
  state.jitStateSize ??= owner.jit_state_size();
  return state.jitStateSize;
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
