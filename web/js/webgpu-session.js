import { clearWebGpuCanvas } from "./webgpu-canvas-clear.js?v=20260904-virgl-gpu-readback-r1";
import { canvasConfiguration } from "./webgpu-readback.js?v=20260904-virgl-gpu-readback-r1";

export class WebGpuSession {
  #adapter;
  #adapterInfo = emptyAdapterInfo();
  #canvas;
  #context;
  #contextClaimed = false;
  #configuredGeneration = 0;
  #device;
  #error = "";
  #format;
  #generation = 0;
  #initPromise;
  #navigator;
  #onChange;
  #revision = 0;
  #state = "idle";

  constructor(canvas, { navigator = globalThis.navigator, onChange = () => {} } = {}) {
    this.#canvas = canvas;
    this.#navigator = navigator;
    this.#onChange = onChange;
  }

  get contextClaimed() {
    return this.#contextClaimed;
  }

  get error() {
    return this.#error;
  }

  async acquire() {
    if (this.#state === "ready") return this.#snapshot();
    if (this.#state === "unavailable" && !this.#contextClaimed) return undefined;
    if (!this.#initPromise) {
      const nextState = this.#contextClaimed ? "recovering" : "initializing";
      if (this.#state !== nextState) {
        this.#state = nextState;
        this.#notify();
      }
      const promise = this.#initialize(this.#revision);
      const tracked = promise.finally(() => {
        if (this.#initPromise === tracked) this.#initPromise = undefined;
      });
      this.#initPromise = tracked;
    }
    await this.#initPromise;
    return this.#state === "ready" ? this.#snapshot() : undefined;
  }

  configure(width = this.#canvas.width, height = this.#canvas.height) {
    if (this.#state !== "ready") throw new Error("WebGPU session is not ready");
    if (
      this.#configuredGeneration === this.#generation &&
      this.#canvas.width === width &&
      this.#canvas.height === height
    ) return;
    if (this.#canvas.width !== width) this.#canvas.width = width;
    if (this.#canvas.height !== height) this.#canvas.height = height;
    this.#context.configure(canvasConfiguration(this.#device, this.#format));
    this.#configuredGeneration = this.#generation;
  }

  clear() {
    return this.#state === "ready" && clearWebGpuCanvas(this.#device, this.#context);
  }

  destroy() {
    this.#revision += 1;
    this.#initPromise = undefined;
    const device = this.#device;
    this.#device = undefined;
    this.#adapter = undefined;
    this.#adapterInfo = emptyAdapterInfo();
    this.#configuredGeneration = 0;
    this.#error = "";
    this.#state = "idle";
    device?.destroy?.();
    this.#notify();
  }

  async #initialize(revision) {
    const gpu = this.#navigator?.gpu;
    if (!gpu) return this.#fail(new Error("WebGPU is not available"));
    try {
      const adapter = await gpu.requestAdapter({ powerPreference: "high-performance" });
      if (revision !== this.#revision) return;
      if (!adapter) throw new Error("No WebGPU adapter is available");
      const device = await adapter.requestDevice();
      if (revision !== this.#revision) {
        device.destroy?.();
        return;
      }
      const context = this.#canvas.getContext("webgpu");
      if (!context) {
        device.destroy?.();
        throw new Error("Canvas WebGPU context is not available");
      }
      this.#adapter = adapter;
      this.#adapterInfo = adapterDiagnostics(adapter);
      this.#device = device;
      this.#context = context;
      this.#contextClaimed = true;
      this.#format = gpu.getPreferredCanvasFormat?.() ?? "bgra8unorm";
      this.#generation += 1;
      this.#error = "";
      this.#state = "ready";
      this.configure();
      device.lost?.then((info) => this.#handleLoss(device, info));
      this.#notify();
    } catch (error) {
      if (revision === this.#revision) this.#fail(error);
    }
  }

  #fail(error) {
    const device = this.#device;
    this.#device = undefined;
    this.#adapter = undefined;
    device?.destroy?.();
    this.#error = error?.message ?? String(error);
    this.#state = "unavailable";
    this.#notify();
  }

  #handleLoss(device, info) {
    if (device !== this.#device) return;
    this.#device = undefined;
    this.#state = "recovering";
    this.#error = `WebGPU device lost${info?.message ? `: ${info.message}` : ""}`;
    this.#initPromise = undefined;
    this.#notify();
  }

  #snapshot() {
    return {
      adapter: this.#adapter,
      adapterClass: this.#adapterInfo.isFallbackAdapter === true
        ? "fallback"
        : this.#adapterInfo.isFallbackAdapter === false ? "non-fallback" : "unknown",
      adapterInfo: { ...this.#adapterInfo },
      canvas: this.#canvas,
      canvasContext: this.#context,
      device: this.#device,
      deviceGeneration: this.#generation,
      format: this.#format,
      isFallbackAdapter: this.#adapterInfo.isFallbackAdapter,
    };
  }

  #notify() {
    this.#onChange({
      adapterInfo: { ...this.#adapterInfo },
      error: this.#error,
      generation: this.#generation,
      state: this.#state,
    });
  }
}

function adapterDiagnostics(adapter) {
  const info = adapter.info ?? {};
  return {
    architecture: safeText(info.architecture),
    description: safeText(info.description, 256),
    isFallbackAdapter: info.isFallbackAdapter ?? adapter.isFallbackAdapter,
    vendor: safeText(info.vendor),
  };
}

function emptyAdapterInfo() {
  return { architecture: "", description: "", isFallbackAdapter: undefined, vendor: "" };
}

function safeText(value, limit = 128) {
  return typeof value === "string" ? value.slice(0, limit) : "";
}
