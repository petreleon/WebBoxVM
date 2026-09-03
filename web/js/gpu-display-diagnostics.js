export class GpuDisplayDiagnostics {
  #adapter = "unknown";
  #adapterInfo = {};
  #backend = "none";
  #canvas;
  #deviceGeneration = 0;
  #frames2d = 0;
  #frames3d = 0;
  #lastError = "";
  #lastSequence;
  #status;
  #threeD = "inactive";
  #threeDDraws = 0;
  #threeDErrors = 0;
  #threeDLastError = "";
  #uploads2d = 0;

  constructor(canvas, status) {
    this.#canvas = canvas;
    this.#status = status;
    this.status("Waiting for guest display");
  }

  get threeDAcceleration() {
    return this.#threeD;
  }

  sessionChanged({ adapterInfo = {}, error = "", generation = 0, state }) {
    this.#adapterInfo = adapterInfo;
    this.#deviceGeneration = generation;
    const fallback = adapterInfo.isFallbackAdapter;
    this.#adapter = fallback === true ? "fallback" : fallback === false ? "non-fallback" : "unknown";
    if (state === "ready") this.#backend = "webgpu";
    else if (state === "recovering") {
      this.#backend = "recovering";
      if (this.#threeD === "webgpu-experimental-capset") this.#threeD = "recovering";
    } else if (state === "initializing") this.#backend = "initializing";
    else if (state === "unavailable") this.#backend = "unavailable";
    else if (state === "idle") {
      this.#backend = "none";
      this.status("Waiting for guest display");
      return;
    }
    if (error) this.#lastError = error;
    this.#sync();
  }

  canvasFallback(error) {
    this.#backend = "canvas2d";
    this.#adapter = "none";
    this.#lastError = error;
    this.status("Canvas2D 2D scanout fallback");
  }

  received2d() {
    this.#frames2d += 1;
    this.#sync();
  }

  uploaded2d(width, height, webgpu) {
    this.#uploads2d += 1;
    const adapter = this.#adapter === "fallback" ? " · fallback adapter" :
      this.#adapter === "non-fallback" ? " · non-fallback adapter" : "";
    this.status(`${webgpu ? "WebGPU" : "Canvas2D"} 2D scanout${adapter} · ${width}×${height}`);
  }

  received3d(sequence) {
    this.#frames3d += 1;
    this.#lastSequence = sequence;
    this.#sync();
  }

  drew3d(frame) {
    this.#threeDDraws += 1;
    this.#threeD = "webgpu-experimental-capset";
    this.status(
      `WebGPU experimental guest 3D · ${frame.canvasWidth}×${frame.canvasHeight} · sequence ${frame.sequence}`,
    );
  }

  error3d(error, prefix) {
    this.#threeDErrors += 1;
    this.#threeDLastError = errorText(error);
    this.#lastError = this.#threeDLastError;
    this.#threeD = "error";
    this.status(`${prefix}: ${this.#threeDLastError}`, "error");
  }

  error2d(error, prefix) {
    this.#lastError = errorText(error);
    this.status(`${prefix}: ${this.#lastError}`, "error");
  }

  reset() {
    this.#frames2d = 0;
    this.#uploads2d = 0;
    this.#frames3d = 0;
    this.#threeDDraws = 0;
    this.#threeDErrors = 0;
    this.#threeDLastError = "";
    this.#lastSequence = undefined;
    this.#lastError = "";
    this.#threeD = "inactive";
    this.status(`Waiting for guest display${this.#backend === "none" ? "" : ` (${this.#backend} ready)`}`);
  }

  status(text, tone = "") {
    if (this.#status) {
      if (this.#status.textContent !== text) this.#status.textContent = text;
      if (tone) this.#status.dataset.tone = tone;
      else delete this.#status.dataset.tone;
    }
    this.#sync();
  }

  #sync() {
    for (const probe of [this.#status, this.#canvas]) {
      if (!probe?.dataset) continue;
      probe.dataset.backend = this.#backend;
      probe.dataset.accelerationPath = "2d-scanout";
      probe.dataset.threeDAcceleration = this.#threeD;
      probe.dataset.threeDCapsetId = "7";
      probe.dataset.adapter = this.#adapter;
      probe.dataset.fallbackAdapter = this.#adapter === "fallback" ? "true" :
        this.#adapter === "non-fallback" ? "false" : "unknown";
      probe.dataset.adapterVendor = this.#adapterInfo.vendor ?? "";
      probe.dataset.adapterArchitecture = this.#adapterInfo.architecture ?? "";
      probe.dataset.adapterDescription = this.#adapterInfo.description ?? "";
      probe.dataset.deviceGeneration = String(this.#deviceGeneration);
      probe.dataset.framesReceived = String(this.#frames2d);
      probe.dataset.uploads = String(this.#uploads2d);
      probe.dataset.threeDFramesReceived = String(this.#frames3d);
      probe.dataset.threeDDraws = String(this.#threeDDraws);
      probe.dataset.threeDErrors = String(this.#threeDErrors);
      probe.dataset.threeDLastError = this.#threeDLastError;
      probe.dataset.threeDLastSequence = this.#lastSequence === undefined ? "" : String(this.#lastSequence);
      probe.dataset.lastError = this.#lastError;
    }
  }
}

function errorText(error) {
  return error?.message ?? String(error);
}
