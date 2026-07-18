export class BootPhaseTimer {
  #last;
  #now;
  #phases = {};
  #startedAt;

  constructor(now = () => performance.now()) {
    this.#now = now;
    this.#startedAt = now();
    this.#last = this.#startedAt;
  }

  end(name) {
    const current = this.#now();
    this.#phases[name] = Math.max(0, current - this.#last);
    this.#last = current;
  }

  finish() {
    return {
      ...this.#phases,
      totalMs: Math.max(0, this.#now() - this.#startedAt),
    };
  }
}
