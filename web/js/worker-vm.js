import { transferableBytes } from "./worker-vm/bytes.js";
import { versionedUrl } from "./asset-version.js";
import { WorkerChannel } from "./worker-vm/channel.js";

function versionedWorkerUrl() {
  return versionedUrl("./vm-worker.js", import.meta.url);
}

export class WorkerVm {
  onAutosave = () => {};
  onError = () => {};
  onMetrics = () => {};
  onNetwork = () => {};
  onUart = () => {};

  #channel;
  #networkStats = {
    rxPackets: 0n,
    status: "offline",
    txPackets: 0n,
    txPending: 0,
  };

  constructor() {
    this.#channel = new WorkerChannel(versionedWorkerUrl(), {
      onAutosave: () => this.onAutosave(),
      onError: (error) => this.onError(error),
      onMetrics: () => this.onMetrics(),
      onNetwork: (status) => this.onNetwork(status),
      onUart: (output) => this.onUart(output),
    });
  }

  boot_iso_with_disk(isoImage, numCores, diskSizeBytes) {
    const bytes = transferableBytes(isoImage);
    return this.#channel
      .request("bootIsoWithDisk", { diskSizeBytes, isoImage: bytes, numCores }, [bytes.buffer])
      .then(({ result }) => result);
  }

  boot_installed_disk(snapshot, numCores, extraBootargs = "") {
    const bytes = transferableBytes(snapshot);
    return this.#channel
      .request("bootInstalledDisk", { diskSnapshot: bytes, extraBootargs, numCores }, [bytes.buffer])
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

  current_instruction(coreId = 0) {
    return this.#channel.request("currentInstruction", { coreId });
  }

  debug_translate_va(va, coreId = 0) {
    return this.#channel.request("debugTranslateVa", { coreId, va: BigInt(va) });
  }

  debug_read_va_u64(va, coreId = 0) {
    return this.#channel.request("debugReadVaU64", { coreId, va: BigInt(va) });
  }

  debug_read_pa_u64(pa) {
    return this.#channel.request("debugReadPaU64", { pa: BigInt(pa) });
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

  network_stats() {
    this.#networkStats.rxPackets = this.#channel.metrics.networkRxPackets;
    this.#networkStats.status = this.#channel.metrics.networkStatus;
    this.#networkStats.txPackets = this.#channel.metrics.networkTxPackets;
    this.#networkStats.txPending = this.#channel.metrics.networkTxPending;
    return this.#networkStats;
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
