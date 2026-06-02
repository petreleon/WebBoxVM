import { clamp } from "./utils.js";

const DEFAULT_STEP_SLICE = 1_000_000;

export class VmRunner {
  #els;
  #term;
  #ui;
  #disk;
  #getEmulator;
  #saveDisk;
  #handleError;
  #running = false;
  #boundEmulator;

  constructor({ els, term, ui, disk, getEmulator, saveDisk, handleError }) {
    this.#els = els;
    this.#term = term;
    this.#ui = ui;
    this.#disk = disk;
    this.#getEmulator = getEmulator;
    this.#saveDisk = saveDisk;
    this.#handleError = handleError;
    this.#els.stepSlice.addEventListener("input", () => {
      this.#getEmulator()?.set_step_slice(this.#stepSlice());
    });
  }

  start() {
    this.#running = true;
    const emulator = this.#bindCurrentEmulator();
    emulator?.start(this.#stepSlice());
  }

  resume() {
    this.#running = true;
    const emulator = this.#bindCurrentEmulator();
    emulator?.resume(this.#stepSlice());
  }

  pause() {
    this.#running = false;
    this.#getEmulator()?.pause();
  }

  stop() {
    this.#running = false;
    this.#getEmulator()?.stop();
  }

  #bindCurrentEmulator() {
    const emulator = this.#getEmulator();
    if (!emulator || emulator === this.#boundEmulator) {
      return emulator;
    }

    emulator.onAutosave = () => this.#saveDisk({ quiet: true }).catch(this.#handleError);
    emulator.onError = (error) => {
      this.#running = false;
      this.#handleError(error);
    };
    emulator.onMetrics = () => this.#ui.updateMetrics(emulator, this.#disk);
    emulator.onUart = (output) => this.#writeUart(output);
    this.#boundEmulator = emulator;
    return emulator;
  }

  #writeUart(output) {
    if (!output) {
      return;
    }
    this.#term.write(output);
    if (this.#els.autoScroll.checked) {
      this.#term.scrollToBottom();
    }
  }

  #stepSlice() {
    return clamp(Number(this.#els.stepSlice.value) || DEFAULT_STEP_SLICE, 1000, 1000000);
  }
}
