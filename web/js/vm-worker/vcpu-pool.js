import { versionedUrl } from "../asset-version.js?v=20260904-virgl-depth-batch-r1";
import { DEFAULT_REQUEST_TIMEOUT_MS, WorkerSlot } from "./vcpu-worker-slot.js?v=20260904-virgl-depth-batch-r1";

const DEFAULT_STOP_TIMEOUT_MS = 1_000;

export class VcpuPool {
  static async create(coreCount, threadedWasm, options = {}) {
    const requestTimeoutMs = options.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
    const workerUrl = versionedUrl("../vcpu-worker.js", import.meta.url);
    const slots = [];
    const pool = new VcpuPool(slots, threadedWasm, options.stopTimeoutMs);
    try {
      for (let core = 0; core < coreCount; core += 1) {
        const worker = new Worker(workerUrl, {
          name: `webbox-vcpu-${core}`,
          type: "module",
        });
        slots.push(new WorkerSlot(worker, core, requestTimeoutMs));
      }
      await Promise.all(
        slots.map((slot) =>
          slot.request({
            glueUrl: threadedWasm.glueUrl,
            memory: threadedWasm.memory,
            module: threadedWasm.module,
            stackSize: 4 * 65536,
            type: "init",
          }),
        ),
      );
      return pool;
    } catch (error) {
      await pool.stop();
      throw error;
    }
  }

  constructor(slots, threadedWasm, stopTimeoutMs = DEFAULT_STOP_TIMEOUT_MS) {
    this.activeToken = undefined;
    this.cancelledToken = undefined;
    this.finishParallelRun = threadedWasm.finishParallelRun;
    this.cancelParallelRun = threadedWasm.cancelParallelRun;
    this.slots = slots;
    this.inFlight = undefined;
    this.stopPromise = undefined;
    this.stopTimeoutMs = stopTimeoutMs;
    this.stopping = false;
  }

  isReady(coreCount) {
    return (
      !this.stopping &&
      this.slots.length === coreCount &&
      this.slots.every((slot) => !slot.dead)
    );
  }

  runRound(emulator, maxSteps) {
    if (this.inFlight) {
      return this.inFlight;
    }
    if (this.stopping || this.slots.length === 0) {
      return Promise.reject(new Error("vCPU worker pool is stopped"));
    }
    const dead = this.slots.find((slot) => slot.dead);
    if (dead) {
      return Promise.reject(new Error(`vCPU ${dead.core} worker is unavailable`));
    }
    this.inFlight = this.executeRound(emulator, maxSteps)
      .finally(() => {
        this.activeToken = undefined;
        this.inFlight = undefined;
      });
    return this.inFlight;
  }

  interrupt() {
    if (this.activeToken === undefined) {
      return false;
    }
    return this.cancel(this.activeToken) === undefined;
  }

  async executeRound(emulator, maxSteps) {
    let began = false;
    let failure;
    let summary;
    try {
      const generation = emulator.parallel_begin_kernel(maxSteps);
      began = true;
      this.activeToken = generation;
      this.cancelledToken = undefined;
      const requests = this.slots.map((slot) =>
        slot.request({ token: generation, type: "run" }, undefined, () => {
          this.cancel(generation);
        }).catch((error) => {
          if (!failure) {
            failure = error;
            this.cancel(generation);
          }
          throw error;
        }),
      );
      await Promise.allSettled(requests);
    } catch (error) {
      failure ||= error;
      if (began) {
        const cancelError = this.cancel(this.activeToken);
        failure ||= cancelError;
      }
    } finally {
      if (began) {
        try {
          summary = this.finishParallelRun(this.activeToken);
        } catch (error) {
          failure ||= error;
        }
      }
    }
    if (failure) throw failure;
    return summary;
  }

  stop() {
    if (this.stopPromise) return this.stopPromise;
    this.stopping = true;
    this.cancel(this.activeToken);
    this.stopPromise = this.stopWorkers();
    return this.stopPromise;
  }

  async stopWorkers() {
    await this.inFlight?.catch(() => {});
    const slots = this.slots;
    this.slots = [];
    await Promise.allSettled(
      slots.map(async (slot) => {
        try {
          if (!slot.dead) {
            await slot.request({ type: "stop" }, this.stopTimeoutMs);
          }
        } finally {
          slot.terminate();
        }
      }),
    );
  }

  cancel(token) {
    if (token === undefined || token === this.cancelledToken) return;
    this.cancelledToken = token;
    try {
      this.cancelParallelRun(token);
    } catch (error) {
      return error;
    }
  }
}
