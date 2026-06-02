const INITIAL_METRICS = {
  allocatedPages: 0,
  installDiskAllocatedBytes: 0n,
  installDiskGeneration: 0n,
  installDiskSizeBytes: 0n,
  pc: 0n,
  totalSteps: 0n,
  uartOutputLen: 0,
};

export class WorkerVm {
  onAutosave = () => {};
  onError = () => {};
  onMetrics = () => {};
  onUart = () => {};

  #worker;
  #nextRequestId = 1;
  #pending = new Map();
  #metrics = { ...INITIAL_METRICS };

  constructor() {
    this.#worker = new Worker(new URL("./vm-worker.js", import.meta.url), { type: "module" });
    this.#worker.addEventListener("message", (event) => this.#handleMessage(event.data));
    this.#worker.addEventListener("error", (event) => {
      this.#rejectAll(event.error ?? new Error(event.message));
      this.onError(event.error ?? new Error(event.message));
    });
  }

  boot_iso_with_disk(isoImage, numCores, diskSizeBytes) {
    const bytes = transferableBytes(isoImage);
    return this.#request(
      "bootIsoWithDisk",
      { diskSizeBytes, isoImage: bytes, numCores },
      [bytes.buffer],
    ).then(({ result }) => result);
  }

  restore_install_disk(snapshot) {
    const bytes = transferableBytes(snapshot);
    return this.#request("restoreInstallDisk", { snapshot: bytes }, [bytes.buffer]).then(
      ({ result }) => result,
    );
  }

  install_disk_snapshot() {
    return this.#request("installDiskSnapshot").then(({ snapshot }) => snapshot);
  }

  send_uart_input(input) {
    this.#post("sendUartInput", { input });
  }

  send_uart_bytes(input) {
    const bytes = transferableBytes(input);
    this.#post("sendUartBytes", { input: bytes }, [bytes.buffer]);
  }

  start(stepSlice) {
    this.#post("start", { stepSlice });
  }

  resume(stepSlice) {
    this.#post("resume", { stepSlice });
  }

  pause() {
    this.#post("pause");
  }

  stop() {
    this.#post("stop");
  }

  set_step_slice(stepSlice) {
    this.#post("setStepSlice", { stepSlice });
  }

  free() {
    this.#post("free");
    this.#worker.terminate();
    this.#rejectAll(new Error("Worker VM terminated"));
  }

  allocated_pages() {
    return this.#metrics.allocatedPages;
  }

  install_disk_allocated_bytes() {
    return this.#metrics.installDiskAllocatedBytes;
  }

  install_disk_generation() {
    return this.#metrics.installDiskGeneration;
  }

  install_disk_size_bytes() {
    return this.#metrics.installDiskSizeBytes;
  }

  pc() {
    return this.#metrics.pc;
  }

  total_steps() {
    return this.#metrics.totalSteps;
  }

  uart_output_len() {
    return this.#metrics.uartOutputLen;
  }

  #request(type, payload = {}, transfer = []) {
    const id = this.#nextRequestId++;
    const message = { id, payload, type };
    const promise = new Promise((resolve, reject) => {
      this.#pending.set(id, { reject, resolve });
    });
    this.#worker.postMessage(message, transfer);
    return promise;
  }

  #post(type, payload = {}, transfer = []) {
    this.#worker.postMessage({ payload, type }, transfer);
  }

  #handleMessage(message) {
    if (message.event) {
      this.#handleEvent(message);
      return;
    }

    const pending = this.#pending.get(message.id);
    if (!pending) {
      return;
    }
    this.#pending.delete(message.id);

    if (message.ok) {
      if (message.value?.metrics) {
        this.#updateMetrics(message.value.metrics);
      }
      pending.resolve(message.value);
    } else {
      pending.reject(new Error(message.error ?? "Worker VM request failed"));
    }
  }

  #handleEvent(message) {
    switch (message.event) {
      case "autosave":
        this.onAutosave();
        break;
      case "error":
        this.onError(new Error(message.error));
        break;
      case "metrics":
        this.#updateMetrics(message.metrics);
        this.onMetrics();
        break;
      case "uart":
        this.onUart(message.output);
        break;
    }
  }

  #updateMetrics(metrics) {
    this.#metrics = {
      allocatedPages: metrics.allocatedPages,
      installDiskAllocatedBytes: metrics.installDiskAllocatedBytes,
      installDiskGeneration: metrics.installDiskGeneration,
      installDiskSizeBytes: metrics.installDiskSizeBytes,
      pc: metrics.pc,
      totalSteps: metrics.totalSteps,
      uartOutputLen: metrics.uartOutputLen,
    };
  }

  #rejectAll(error) {
    for (const pending of this.#pending.values()) {
      pending.reject(error);
    }
    this.#pending.clear();
  }
}

function transferableBytes(bytes) {
  if (bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength) {
    return bytes;
  }
  return bytes.slice();
}
