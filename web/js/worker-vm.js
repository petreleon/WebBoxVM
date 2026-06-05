import { transferableBytes } from "./worker-vm/bytes.js";
import { WorkerChannel } from "./worker-vm/channel.js";

export class WorkerVm {
  onAutosave = () => {};
  onError = () => {};
  onMetrics = () => {};
  onUart = () => {};

  #channel;

  constructor() {
    this.#channel = new WorkerChannel(new URL("./vm-worker.js", import.meta.url), {
      onAutosave: () => this.onAutosave(),
      onError: (error) => this.onError(error),
      onMetrics: () => this.onMetrics(),
      onUart: (output) => this.onUart(output),
    });
  }

  boot_iso_with_disk(isoImage, numCores, diskSizeBytes) {
    const bytes = transferableBytes(isoImage);
    return this.#channel
      .request("bootIsoWithDisk", { diskSizeBytes, isoImage: bytes, numCores }, [bytes.buffer])
      .then(({ result }) => result);
  }

  restore_install_disk(snapshot) {
    const bytes = transferableBytes(snapshot);
    return this.#channel
      .request("restoreInstallDisk", { snapshot: bytes }, [bytes.buffer])
      .then(({ result }) => result);
  }

  install_disk_snapshot() {
    return this.#channel.request("installDiskSnapshot").then(({ snapshot }) => snapshot);
  }

  compile_jit_block(coreId = 0) {
    return this.#channel.request("compileJitBlock", { coreId });
  }

  run_jit_block(coreId = 0) {
    return this.#channel.request("runJitBlock", { coreId });
  }

  send_uart_input(input) {
    this.#channel.post("sendUartInput", { input });
  }

  send_uart_bytes(input) {
    const bytes = transferableBytes(input);
    this.#channel.post("sendUartBytes", { input: bytes }, [bytes.buffer]);
  }

  start(stepSlice) {
    this.#channel.post("start", { stepSlice });
  }

  resume(stepSlice) {
    this.#channel.post("resume", { stepSlice });
  }

  pause() {
    this.#channel.post("pause");
  }

  stop() {
    this.#channel.post("stop");
  }

  set_step_slice(stepSlice) {
    this.#channel.post("setStepSlice", { stepSlice });
  }

  set_jit_enabled(enabled) {
    this.#channel.post("setJitEnabled", { enabled });
  }

  free() {
    this.#channel.free();
  }

  allocated_pages() {
    return this.#channel.metrics.allocatedPages;
  }

  install_disk_allocated_bytes() {
    return this.#channel.metrics.installDiskAllocatedBytes;
  }

  install_disk_generation() {
    return this.#channel.metrics.installDiskGeneration;
  }

  install_disk_size_bytes() {
    return this.#channel.metrics.installDiskSizeBytes;
  }

  jit_stats() {
    return this.#channel.metrics.jitStats;
  }

  pc() {
    return this.#channel.metrics.pc;
  }

  total_steps() {
    return this.#channel.metrics.totalSteps;
  }

  uart_output_len() {
    return this.#channel.metrics.uartOutputLen;
  }
}
