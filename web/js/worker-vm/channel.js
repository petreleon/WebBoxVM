const INITIAL_METRICS = {
  allocatedPages: 0,
  installDiskAllocatedBytes: 0n,
  installDiskGeneration: 0n,
  installDiskSizeBytes: 0n,
  jitStats: { cacheBlocks: 0, enabled: true, hitSites: 0, recentRejects: [], rejectedBlocks: 0 },
  pc: 0n,
  totalSteps: 0n,
  uartOutputLen: 0,
};

export class WorkerChannel {
  #callbacks;
  #metrics = { ...INITIAL_METRICS };
  #nextRequestId = 1;
  #pending = new Map();
  #worker;

  constructor(workerUrl, callbacks) {
    this.#callbacks = callbacks;
    this.#worker = new Worker(workerUrl, { type: "module" });
    this.#worker.addEventListener("message", (event) => this.#handleMessage(event.data));
    this.#worker.addEventListener("error", (event) => {
      const error = event.error ?? new Error(event.message);
      this.#rejectAll(error);
      this.#callbacks.onError(error);
    });
  }

  get metrics() {
    return this.#metrics;
  }

  request(type, payload = {}, transfer = []) {
    const id = this.#nextRequestId++;
    const message = { id, payload, type };
    const promise = new Promise((resolve, reject) => {
      this.#pending.set(id, { reject, resolve });
    });
    this.#worker.postMessage(message, transfer);
    return promise;
  }

  post(type, payload = {}, transfer = []) {
    this.#worker.postMessage({ payload, type }, transfer);
  }

  free() {
    this.post("free");
    this.#worker.terminate();
    this.#rejectAll(new Error("Worker VM terminated"));
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
        this.#callbacks.onAutosave();
        break;
      case "error":
        this.#callbacks.onError(new Error(message.error));
        break;
      case "metrics":
        this.#updateMetrics(message.metrics);
        this.#callbacks.onMetrics();
        break;
      case "uart":
        this.#callbacks.onUart(message.output);
        break;
    }
  }

  #updateMetrics(metrics) {
    this.#metrics = {
      allocatedPages: metrics.allocatedPages,
      installDiskAllocatedBytes: metrics.installDiskAllocatedBytes,
      installDiskGeneration: metrics.installDiskGeneration,
      installDiskSizeBytes: metrics.installDiskSizeBytes,
      jitStats: metrics.jitStats ?? INITIAL_METRICS.jitStats,
      pc: metrics.pc,
      totalSteps: metrics.totalSteps,
      uartOutputLen: metrics.uartOutputLen,
    };
    const jitProbe = document.querySelector("[data-testid='webboxvm-jit-stats']");
    if (jitProbe) {
      jitProbe.textContent = JSON.stringify(this.#metrics.jitStats ?? null);
    }
  }

  #rejectAll(error) {
    for (const pending of this.#pending.values()) {
      pending.reject(error);
    }
    this.#pending.clear();
  }
}
