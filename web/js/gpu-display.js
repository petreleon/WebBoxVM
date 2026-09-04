import { CanvasScanoutRenderer } from "./canvas-scanout.js?v=20260904-virgl-material-batch-r1";
import {
  extractGpu3dSequence,
  parseGpu3dPacket,
} from "./gpu-3d-packet.js?v=20260904-virgl-material-batch-r1";
import { GpuDisplayDiagnostics } from "./gpu-display-diagnostics.js?v=20260904-virgl-material-batch-r1";
import { parseGpuScanoutPacket } from "./gpu-scanout-packet.js?v=20260904-virgl-material-batch-r1";
import { GpuScanoutState } from "./gpu-scanout-state.js?v=20260904-virgl-material-batch-r1";
import { ExperimentalWebGpu3dRenderer } from "./webgpu-3d.js?v=20260904-virgl-material-batch-r1";
import { WebGpuScanoutRenderer } from "./webgpu-scanout.js?v=20260904-virgl-material-batch-r1";
import { WebGpuSession } from "./webgpu-session.js?v=20260904-virgl-material-batch-r1";

export { extractGpu3dSequence, parseGpu3dPacket }
  from "./gpu-3d-packet.js?v=20260904-virgl-material-batch-r1";
export { padBgraRows, paddedBytesPerRow, parseGpuScanoutPacket }
  from "./gpu-scanout-packet.js?v=20260904-virgl-material-batch-r1";

export class GuestDisplay {
  #canvas2d;
  #diagnostics;
  #epoch = 0;
  #flushPromise;
  #gpu3d;
  #gpu3dPromise;
  #presentationClaim = 0;
  #presentationMode = "scanout";
  #scanout;
  #session;
  #state = new GpuScanoutState();

  constructor(canvas, status, options = {}) {
    if (!canvas) throw new Error("Guest display canvas is required");
    this.#diagnostics = new GpuDisplayDiagnostics(canvas, status);
    this.#session = new WebGpuSession(canvas, {
      navigator: options.navigator,
      onChange: (info) => this.#sessionChanged(info),
    });
    this.#scanout = new WebGpuScanoutRenderer(this.#session, options.textureUsage);
    this.#gpu3d = new ExperimentalWebGpu3dRenderer(this.#session, options);
    this.#canvas2d = new CanvasScanoutRenderer(canvas);
  }

  present(packet) {
    try {
      this.#state.apply(parseGpuScanoutPacket(packet));
    } catch (error) {
      this.#diagnostics.error2d(error, "Invalid guest display frame");
      return Promise.resolve(false);
    }
    this.#diagnostics.received2d();
    if (this.#presentationMode !== "scanout") return Promise.resolve(false);
    return this.#scheduleScanout();
  }

  present3d(packet) {
    let frame;
    try {
      frame = parseGpu3dPacket(packet);
    } catch (error) {
      this.#diagnostics.error3d(error, "Invalid guest 3D frame");
      return Promise.resolve({ sequence: extractGpu3dSequence(packet), success: false });
    }
    this.#diagnostics.received3d(frame.sequence);
    const claim = ++this.#presentationClaim;
    this.#presentationMode = "guest-3d-pending";
    const epoch = this.#epoch;
    const previous = this.#gpu3dPromise ?? Promise.resolve();
    const scheduled = previous.then(() => this.#draw3d(frame, epoch));
    const tracked = scheduled
      .then((result) => this.#settle3d(result, claim))
      .catch((error) => {
        if (epoch === this.#epoch && claim === this.#presentationClaim) {
          this.#diagnostics.error3d(error, "Guest 3D draw failed");
        }
        return this.#settle3d({ sequence: frame.sequence, success: false }, claim);
      })
      .finally(() => {
        if (this.#gpu3dPromise === tracked) this.#gpu3dPromise = undefined;
      });
    this.#gpu3dPromise = tracked;
    return tracked;
  }

  async acquireWebGpuBackend() {
    const backend = await this.#session.acquire();
    return backend ? { ...backend, scanoutTexture: this.#scanout.texture } : undefined;
  }

  reset() {
    this.#epoch += 1;
    this.#presentationClaim += 1;
    this.#presentationMode = "scanout";
    this.#state.reset();
    this.#session.clear();
    this.#scanout.reset();
    this.#gpu3d.invalidate();
    this.#canvas2d.reset();
    this.#diagnostics.reset();
  }

  destroy() {
    this.reset();
    this.#session.destroy();
  }

  async whenIdle() {
    while (this.#flushPromise || this.#gpu3dPromise) {
      await Promise.all([this.#flushPromise, this.#gpu3dPromise].filter(Boolean));
    }
  }

  #scheduleScanout() {
    if (this.#presentationMode !== "scanout") return Promise.resolve(false);
    if (this.#flushPromise) return this.#flushPromise;
    const epoch = this.#epoch;
    const tracked = Promise.resolve().then(() => this.#flushScanout(epoch))
      .catch((error) => this.#diagnostics.error2d(error, "Guest display update failed"))
      .finally(() => {
        if (this.#flushPromise === tracked) this.#flushPromise = undefined;
        if (this.#presentationMode === "scanout" && this.#state.dirty) this.#scheduleScanout();
      });
    this.#flushPromise = tracked;
    return tracked;
  }

  async #flushScanout(epoch) {
    const backend = await this.#session.acquire();
    if (epoch !== this.#epoch || this.#presentationMode !== "scanout" || !this.#state.pixels) {
      return false;
    }
    const dirty = this.#state.takeDirty();
    if (!dirty) return false;
    if (backend) {
      this.#scanout.render(backend, this.#state, dirty);
      this.#diagnostics.uploaded2d(this.#state.width, this.#state.height, true);
    } else if (!this.#session.contextClaimed) {
      this.#canvas2d.render(this.#state, dirty);
      this.#diagnostics.canvasFallback(this.#session.error);
      this.#diagnostics.uploaded2d(this.#state.width, this.#state.height, false);
    } else {
      throw new Error(this.#session.error || "WebGPU recovery is unavailable");
    }
    return true;
  }

  async #draw3d(frame, epoch) {
    const backend = await this.#session.acquire();
    if (epoch !== this.#epoch) return { sequence: frame.sequence, success: false };
    if (!backend) throw new Error("Experimental guest 3D requires WebGPU; Canvas2D is 2D-only");
    const rendered = await this.#gpu3d.render(backend, frame, () => epoch === this.#epoch);
    if (!rendered || epoch !== this.#epoch) {
      return { sequence: frame.sequence, success: false };
    }
    this.#diagnostics.drew3d(frame);
    return { sequence: frame.sequence, success: true };
  }

  #settle3d(result, claim) {
    if (claim !== this.#presentationClaim) return result;
    if (result.success) {
      this.#presentationMode = "guest-3d-active";
    } else {
      this.#presentationMode = "scanout";
      this.#state.markFull();
      if (this.#state.dirty) this.#scheduleScanout();
    }
    return result;
  }

  #sessionChanged(info) {
    this.#diagnostics.sessionChanged(info);
    if (info.state !== "recovering") return;
    this.#presentationClaim += 1;
    this.#presentationMode = "scanout";
    this.#scanout.invalidate();
    this.#gpu3d.invalidate();
    this.#state.markFull();
    if (this.#state.dirty) this.#scheduleScanout();
  }
}
