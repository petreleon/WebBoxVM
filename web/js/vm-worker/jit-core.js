export function prepareNextJitCore(emulator) {
  if (typeof emulator?.jit_prepare_next_core === "function") {
    return Number(emulator.jit_prepare_next_core());
  }
  // Keep focused single-core unit mocks compatible. Production Wasm exports
  // jit_prepare_next_core so cooperative multicore execution never takes this path.
  return 0;
}

export function pcForCore(emulator, coreId = 0) {
  if (typeof emulator?.pc_for_core === "function") {
    return emulator.pc_for_core(coreId);
  }
  if (coreId === 0 && typeof emulator?.pc === "function") {
    return emulator.pc();
  }
  throw new Error(`Per-core PC is unavailable for vCPU ${coreId}`);
}
