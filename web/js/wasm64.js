export function assertWasm64Supported() {
  if (typeof BigInt !== "function") {
    throw new Error("Wasm64 requires BigInt support in this browser");
  }

  if (!globalThis.WebAssembly?.Memory) {
    throw new Error("WebAssembly memory is unavailable in this browser");
  }

  try {
    new WebAssembly.Memory({ initial: 1n, maximum: 1n, address: "i64" });
  } catch (error) {
    throw new Error(
      `Wasm64 Memory64 is unavailable in this browser: ${error.message}`,
    );
  }
}

export function wasm64ThreadsSupported(scope = globalThis) {
  if (
    scope.crossOriginIsolated !== true ||
    typeof scope.SharedArrayBuffer !== "function" ||
    !scope.Atomics ||
    typeof scope.Worker !== "function" ||
    !scope.WebAssembly?.Memory
  ) {
    return false;
  }
  try {
    const memory = new scope.WebAssembly.Memory({
      address: "i64",
      initial: 1n,
      maximum: 1n,
      shared: true,
    });
    return memory.buffer instanceof scope.SharedArrayBuffer;
  } catch {
    return false;
  }
}
