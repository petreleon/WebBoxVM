import { clamp } from "./utils.js";
import { UartBootTimeline } from "./boot-timeline.js";

const DEFAULT_STEP_SLICE = 5_000_000;
const MAX_STEP_SLICE = 50_000_000;
const UART_TAIL_LIMIT = 32768;

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
  #bootTimeline;
  #uartTail = "";
  #uartProbeText;

  constructor({
    els,
    term,
    ui,
    disk,
    getEmulator,
    saveDisk,
    handleError,
    now,
    onBootTimeline,
  }) {
    this.#els = els;
    this.#term = term;
    this.#ui = ui;
    this.#disk = disk;
    this.#getEmulator = getEmulator;
    this.#saveDisk = saveDisk;
    this.#handleError = handleError;
    this.#bootTimeline = new UartBootTimeline({
      now,
      onMilestone: onBootTimeline,
    });
    this.#els.stepSlice.addEventListener("input", () => {
      this.#getEmulator()?.set_step_slice(this.#stepSlice());
    });
    const { text } = installUartProbe();
    this.#uartProbeText = text;
  }

  start({ installedSystem = false } = {}) {
    this.#running = true;
    this.#beginBoot(installedSystem);
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

    emulator.onAutosave = () => {
      if (!this.#disk.shouldAutosave(emulator)) {
        return;
      }
      this.#saveDisk({ quiet: true }).catch(this.#handleError);
    };
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
    this.#bootTimeline.observe(output);
    if (this.#els.autoScroll.checked) {
      this.#term.scrollToBottom();
    }
  }

  #recordUart(output) {
    if (output.length >= UART_TAIL_LIMIT) {
      this.#uartTail = output.slice(-UART_TAIL_LIMIT);
      this.#uartProbeText.data = this.#uartTail;
      return;
    }
    const overflow = Math.max(0, this.#uartTail.length + output.length - UART_TAIL_LIMIT);
    if (overflow > 0) {
      this.#uartTail = this.#uartTail.slice(overflow);
      this.#uartProbeText.deleteData(0, overflow);
    }
    this.#uartTail += output;
    this.#uartProbeText.appendData(output);
  }

  #beginBoot(installedSystem) {
    if (this.#uartTail) {
      this.#uartTail = "";
      this.#uartProbeText.data = "";
    }
    this.#bootTimeline.start({ installedSystem });
  }

  #stepSlice() {
    return clamp(Number(this.#els.stepSlice.value) || DEFAULT_STEP_SLICE, 1000, MAX_STEP_SLICE);
  }
}

function installUartProbe() {
  const probe = document.createElement("pre");
  const text = document.createTextNode("");
  probe.append(text);
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
  return { probe, text };
}
