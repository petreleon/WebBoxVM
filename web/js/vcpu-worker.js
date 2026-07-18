let runParallelCore;
let wasmExports;

self.onmessage = async ({ data }) => {
  const { core, id, type } = data;
  try {
    if (type === "init") {
      const glue = await import(data.glueUrl);
      wasmExports = glue.initSync({
        memory: data.memory,
        module: data.module,
        thread_stack_size: data.stackSize,
      });
      runParallelCore = glue.run_parallel_core;
      respond(id, true);
      return;
    }
    if (type === "run") {
      runParallelCore(data.token, core);
      respond(id, true);
      return;
    }
    if (type === "stop") {
      wasmExports?.__wbindgen_thread_destroy();
      respond(id, true);
      self.close();
      return;
    }
    throw new Error(`Unknown vCPU worker request: ${type}`);
  } catch (error) {
    respond(id, false, error?.stack ?? String(error));
  }
};

function respond(id, ok, error) {
  self.postMessage(ok ? { id, ok, value: {} } : { error, id, ok });
}
