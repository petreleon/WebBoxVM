export const DEFAULT_REQUEST_TIMEOUT_MS = 60_000;

export class WorkerSlot {
  constructor(worker, core, requestTimeoutMs = DEFAULT_REQUEST_TIMEOUT_MS) {
    this.core = core;
    this.dead = false;
    this.nextId = 1;
    this.pending = new Map();
    this.requestTimeoutMs = requestTimeoutMs;
    this.terminated = false;
    this.worker = worker;
    worker.onmessage = ({ data }) => this.settle(data);
    worker.onerror = (event) => {
      this.fail(event.error ?? new Error(event.message || `vCPU ${core} worker failed`));
    };
    worker.onmessageerror = () => {
      this.fail(new Error(`vCPU ${core} worker sent an unreadable message`));
    };
  }

  request(message, timeoutMs = this.requestTimeoutMs, onTimeout) {
    if (this.dead) {
      return Promise.reject(new Error(`vCPU ${this.core} worker is unavailable`));
    }
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const pending = { reject, resolve, timer: undefined, timeoutError: undefined };
      pending.timer = setTimeout(() => {
        const error = new Error(`vCPU ${this.core} ${message.type} request timed out`);
        if (onTimeout) {
          // A running Rust worker owns a registry lease. Keep the request
          // pending after cancellation so its eventual return drops that lease.
          pending.timeoutError = error;
          pending.timer = undefined;
          onTimeout(error);
          return;
        }
        this.fail(error);
      }, timeoutMs);
      this.pending.set(id, pending);
      try {
        this.worker.postMessage({ ...message, core: this.core, id });
      } catch (error) {
        this.fail(error);
      }
    });
  }

  settle(message) {
    const pending = this.pending.get(message.id);
    if (!pending) return;
    clearTimeout(pending.timer);
    this.pending.delete(message.id);
    if (pending.timeoutError) pending.reject(pending.timeoutError);
    else if (message.ok) pending.resolve(message.value);
    else pending.reject(new Error(message.error || `vCPU ${this.core} request failed`));
  }

  fail(error) {
    if (this.dead) return;
    const failure = error instanceof Error ? error : new Error(String(error));
    this.terminate(failure);
  }

  terminate(error = new Error(`vCPU ${this.core} worker terminated`)) {
    if (this.terminated) return;
    this.terminated = true;
    this.dead = true;
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
    this.worker.terminate();
  }
}
