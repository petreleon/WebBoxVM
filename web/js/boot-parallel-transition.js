export class BootParallelTransition {
  #disk;
  #generation = 0;
  #getEmulator;
  #handleError;
  #loginReady = false;
  #requested = false;
  #secondaryCpuReady = false;
  #ui;

  constructor({ disk, getEmulator, handleError, ui }) {
    this.#disk = disk;
    this.#getEmulator = getEmulator;
    this.#handleError = handleError;
    this.#ui = ui;
  }

  reset() {
    this.#generation += 1;
    this.#loginReady = false;
    this.#requested = false;
    this.#secondaryCpuReady = false;
  }

  observe(milestone) {
    if (milestone.name === "cpu1-online") {
      this.#secondaryCpuReady = true;
    } else if (milestone.name === "login-prompt") {
      this.#loginReady = true;
    }
    if (!this.#secondaryCpuReady || !this.#loginReady || this.#requested) {
      return;
    }
    const generation = this.#generation;
    const emulator = this.#getEmulator();
    if (typeof emulator?.transition_to_parallel !== "function") {
      return;
    }
    this.#requested = true;
    Promise.resolve()
      .then(() => {
        if (this.#isCurrent(emulator, generation)) {
          return emulator.transition_to_parallel();
        }
      })
      .then((result) => this.#report(emulator, generation, result))
      .catch((error) => {
        if (this.#isCurrent(emulator, generation)) {
          this.#handleError(error);
        }
      });
  }

  #isCurrent(emulator, generation) {
    return generation === this.#generation && emulator === this.#getEmulator();
  }

  #report(emulator, generation, result) {
    if (!this.#isCurrent(emulator, generation)) {
      return;
    }
    const mode = result?.executionMode ?? emulator.execution_mode?.();
    if (mode) {
      const reason = result?.reason ? ` (${result.reason})` : "";
      this.#ui.log(`Fast boot execution mode: ${mode}${reason}`);
    }
    this.#ui.updateMetrics(emulator, this.#disk);
  }
}
