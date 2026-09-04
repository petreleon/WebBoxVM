import { initialMetrics } from "./channel-state.js?v=20260904-virgl-depth-texture-color-r1";

export class WorkerChannel {
  #callbacks;
  #closing = false;
  #freePromise;
  #freeTimeoutMs;
  #instructionProbe;
  #jitProbe;
  #metrics = initialMetrics();
  #nextRequestId = 1;
  #pending = new Map();
  #terminated = false;
  #worker;
  constructor(workerUrl, callbacks, options = {}) {
    this.#callbacks = callbacks;
    this.#freeTimeoutMs = options.freeTimeoutMs ?? 2_000;
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
    if (this.#closing) {
      return Promise.reject(new Error("Worker VM is closing"));
    }
    const id = this.#nextRequestId++;
    const message = { id, payload, type };
    const promise = new Promise((resolve, reject) => {
      this.#pending.set(id, { reject, resolve });
    });
    try {
      this.#worker.postMessage(message, transfer);
    } catch (error) {
      this.#pending.get(id)?.reject(error);
      this.#pending.delete(id);
    }
    return promise;
  }
  post(type, payload = {}, transfer = []) {
    if (this.#closing) return;
    this.#worker.postMessage({ payload, type }, transfer);
  }
  free() {
    if (this.#freePromise) return this.#freePromise;
    const acknowledgment = this.request("free").catch(() => {});
    this.#closing = true;
    let timeout;
    this.#freePromise = Promise.race([
      acknowledgment,
      new Promise((resolve) => {
        timeout = setTimeout(resolve, this.#freeTimeoutMs);
      }),
    ]).then(() => {
      clearTimeout(timeout);
      if (!this.#terminated) {
        this.#terminated = true;
        this.#worker.terminate();
      }
      this.#rejectAll(new Error("Worker VM terminated"));
    });
    return this.#freePromise;
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
        if (message.installDiskGeneration !== undefined) {
          this.#metrics.installDiskGeneration = message.installDiskGeneration;
        }
        this.#callbacks.onAutosave();
        break;
      case "error":
        this.#callbacks.onError(new Error(message.error));
        break;
      case "metrics":
        this.#updateMetrics(message.metrics);
        this.#callbacks.onMetrics();
        break;
      case "gpuFrame":
        this.#callbacks.onGpuFrame?.(message.packet);
        break;
      case "gpu3dFrame":
        this.#callbacks.onGpu3dFrame?.(message.packet);
        break;
      case "gpuReset":
        this.#callbacks.onGpuReset?.(message.generation);
        break;
      case "network":
        this.#metrics.networkStatus = message.status;
        this.#callbacks.onNetwork?.(message.status);
        break;
      case "uart":
        this.#callbacks.onUart(message.output);
        break;
    }
  }
  #updateMetrics(metrics) {
    const hasJitStats = metrics.jitStats !== undefined;
    const hasInstruction = Object.hasOwn(metrics, "currentInstruction");
    this.#metrics.allocatedPages = metrics.allocatedPages;
    this.#metrics.cooperativeIdleFastForwardCycles = metrics.cooperativeIdleFastForwardCycles;
    this.#metrics.cooperativeWfeParks = metrics.cooperativeWfeParks;
    this.#metrics.executionMode = metrics.executionMode ?? this.#metrics.executionMode;
    if (hasInstruction) {
      this.#metrics.currentInstruction = metrics.currentInstruction;
    }
    this.#metrics.installDiskAllocatedBytes = metrics.installDiskAllocatedBytes;
    this.#metrics.installDiskGeneration = metrics.installDiskGeneration;
    this.#metrics.installDiskSizeBytes = metrics.installDiskSizeBytes;
    this.#metrics.jitStats = metrics.jitStats ?? this.#metrics.jitStats;
    this.#metrics.networkRxPackets = metrics.networkRxPackets;
    this.#metrics.networkStatus = metrics.networkStatus;
    this.#metrics.networkTxPackets = metrics.networkTxPackets;
    this.#metrics.networkTxPending = metrics.networkTxPending;
    this.#metrics.pc = metrics.pc;
    this.#metrics.totalSteps = metrics.totalSteps;
    this.#metrics.uartOutputLen = metrics.uartOutputLen;
    this.#updateMetricProbes(hasJitStats, hasInstruction);
  }
  #updateMetricProbes(hasJitStats, hasInstruction) {
    if (hasJitStats) {
      this.#jitProbe ||= document.querySelector("[data-testid='webboxvm-jit-stats']");
      if (this.#jitProbe) {
        this.#setProbeText(this.#jitProbe, JSON.stringify(this.#metrics.jitStats ?? null));
      }
    }
    if (hasInstruction) {
      this.#instructionProbe ||= document.querySelector(
        "[data-testid='webboxvm-current-instruction']",
      );
      if (this.#instructionProbe) {
        this.#setProbeText(this.#instructionProbe, this.#metrics.currentInstruction ?? "");
      }
    }
  }
  #setProbeText(probe, text) {
    if (probe.textContent !== text) {
      probe.textContent = text;
    }
  }

  #rejectAll(error) {
    for (const pending of this.#pending.values()) {
      pending.reject(error);
    }
    this.#pending.clear();
  }
}
