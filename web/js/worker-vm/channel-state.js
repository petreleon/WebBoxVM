export function initialMetrics() {
  return {
    allocatedPages: 0,
    cooperativeIdleFastForwardCycles: 0n,
    cooperativeWfeParks: 0n,
    currentInstruction: undefined,
    executionMode: "cooperative",
    installDiskAllocatedBytes: 0n,
    installDiskGeneration: 0n,
    installDiskSizeBytes: 0n,
    jitStats: { cacheBlocks: 0, enabled: true, hitSites: 0, recentRejects: [], rejectedBlocks: 0 },
    networkRxPackets: 0n,
    networkStatus: "offline",
    networkTxPackets: 0n,
    networkTxPending: 0,
    pc: 0n,
    totalSteps: 0n,
    uartOutputLen: 0,
  };
}
