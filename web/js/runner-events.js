export function bindRunnerEvents(emulator, handlers) {
  const current = handlers.current;
  emulator.onAutosave = () => {
    if (current()) handlers.autosave();
  };
  emulator.onError = (error) => {
    if (current()) handlers.error(error);
  };
  emulator.onMetrics = () => {
    if (current()) handlers.metrics();
  };
  emulator.onGpuFrame = (packet) => {
    if (current()) handlers.frame2d(packet);
  };
  emulator.onGpu3dFrame = (packet) => {
    if (!current()) return;
    Promise.resolve(handlers.frame3d(packet)).then((result) => {
      if (current() && result?.sequence !== undefined) {
        result.readback ? emulator.gpu3d_ack?.(result.sequence, result.success, result.readback)
          : emulator.gpu3d_ack?.(result.sequence, result.success);
      }
    });
  };
  emulator.onGpuReset = () => {
    if (current()) handlers.gpuReset();
  };
  emulator.onNetwork = (status) => {
    if (current()) handlers.network(status);
  };
  emulator.onUart = (output) => {
    if (current()) handlers.uart(output);
  };
}
