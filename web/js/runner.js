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
  #pumpScheduled = false;
  #lastUart = 0;

  constructor({ els, term, ui, disk, getEmulator, saveDisk, handleError }) {
    this.#els = els;
    this.#term = term;
    this.#ui = ui;
    this.#disk = disk;
    this.#getEmulator = getEmulator;
    this.#saveDisk = saveDisk;
    this.#handleError = handleError;
  }

  start() {
    this.#running = true;
    this.#lastUart = 0;
    this.#schedulePump();
  }

  resume() {
    this.#running = true;
    this.#schedulePump();
  }

  pause() {
    this.#running = false;
  }

  stop() {
    this.#running = false;
    this.#lastUart = 0;
  }

  #schedulePump() {
    if (!this.#running || this.#pumpScheduled) {
      return;
    }
    this.#pumpScheduled = true;
    requestAnimationFrame(() => this.#runFrame());
  }

  #runFrame() {
    this.#pumpScheduled = false;
    const emulator = this.#getEmulator();
    if (!this.#running || !emulator) {
      return;
    }

    const frameStart = performance.now();
    const stepSlice = clamp(
      Number(this.#els.stepSlice.value) || DEFAULT_STEP_SLICE,
      1000,
      1000000,
    );
    let batches = 0;

    try {
      do {
        emulator.run_kernel(stepSlice);
        this.#drainUart(emulator);
        batches += 1;
      } while (this.#running && performance.now() - frameStart < 24 && batches < 8);

      this.#ui.updateMetrics(emulator, this.#disk);
      this.#scheduleDiskAutosave(emulator);
      this.#schedulePump();
    } catch (error) {
      this.#running = false;
      this.#handleError(error);
    }
  }

  #drainUart(emulator) {
    const output = emulator.uart_output_since(this.#lastUart);
    if (!output) {
      return;
    }
    this.#lastUart = emulator.uart_output_len();
    this.#term.write(output);
    if (this.#els.autoScroll.checked) {
      this.#term.scrollToBottom();
    }
  }

  #scheduleDiskAutosave(emulator) {
    if (this.#disk.shouldAutosave(emulator)) {
      this.#saveDisk({ quiet: true }).catch(this.#handleError);
    }
  }
}
