import { clamp } from "./utils.js";

const DEFAULT_STEP_SLICE = 1_000_000;
const MAX_STEP_SLICE = 50_000_000;

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
  #uartTail = "";
  #uartProbe;

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
    this.#uartProbe = installUartProbe();
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
    emulator.onNetwork = (status) => this.#ui.log(`Network proxy ${status}`);
    emulator.onUart = (output) => this.#writeUart(output);
    this.#boundEmulator = emulator;
    return emulator;
  }

  #writeUart(output) {
    if (!output) {
      return;
    }
    this.#term.write(output);
    this.#recordUart(output);
    if (this.#els.autoScroll.checked) {
      this.#term.scrollToBottom();
    }
  }

  #recordUart(output) {
    this.#uartTail = (this.#uartTail + output).slice(-32768);
    this.#uartProbe.textContent = this.#uartTail;
  }

  #stepSlice() {
    return clamp(Number(this.#els.stepSlice.value) || DEFAULT_STEP_SLICE, 1000, MAX_STEP_SLICE);
  }
}

function installUartProbe() {
  const probe = document.createElement("pre");
  probe.dataset.testid = "webboxvm-uart-tail";
  probe.style.cssText = [
    "position:fixed",
    "left:-10000px",
    "top:0",
    "width:1px",
    "height:1px",
    "overflow:hidden",
  ].join(";");
  document.body.append(probe);
  return probe;
}
