export async function captureWebGpuErrors(device, action) {
  if (
    typeof device?.pushErrorScope !== "function" ||
    typeof device?.popErrorScope !== "function"
  ) {
    throw new Error("WebGPU validation and allocation error tracking is unavailable");
  }

  device.pushErrorScope("out-of-memory");
  device.pushErrorScope("validation");
  let result;
  let actionError;
  let actionFailed = false;
  try {
    result = action();
  } catch (error) {
    actionFailed = true;
    actionError = error;
  }

  // Pop before awaiting so unrelated presenter work cannot enter these scopes.
  const validation = popScope(device);
  const allocation = popScope(device);
  const outcomes = await Promise.allSettled([
    actionFailed ? Promise.reject(actionError) : Promise.resolve(result),
    validation,
    allocation,
  ]);
  const [operation, validationResult, allocationResult] = outcomes;
  if (operation.status === "rejected") throw operation.reason;
  if (validationResult.status === "rejected") throw validationResult.reason;
  if (allocationResult.status === "rejected") throw allocationResult.reason;
  if (allocationResult.value) throw gpuError("allocation", allocationResult.value);
  if (validationResult.value) throw gpuError("validation", validationResult.value);
  return operation.value;
}

function popScope(device) {
  try {
    return Promise.resolve(device.popErrorScope());
  } catch (error) {
    return Promise.reject(error);
  }
}

function gpuError(kind, error) {
  return new Error(`WebGPU ${kind} error: ${error?.message ?? String(error)}`);
}
