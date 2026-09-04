import { clamp } from "./utils.js?v=20260904-virgl-depth-compare-r1";
import { UartBootTimeline } from "./boot-timeline.js?v=20260904-virgl-depth-compare-r1";
import { BootParallelTransition } from "./boot-parallel-transition.js?v=20260904-virgl-depth-compare-r1";
import { installUartProbe } from "./uart-probe.js?v=20260904-virgl-depth-compare-r1";
import { TerminalWriter } from "./terminal-writer.js?v=20260904-virgl-depth-compare-r1";
import { bindRunnerEvents } from "./runner-events.js?v=20260904-virgl-depth-compare-r1";

const DEFAULT_STEP_SLICE = 5_000_000;
const MAX_STEP_SLICE = 50_000_000;
const UART_TAIL_LIMIT = 32768;

export class VmRunner {
  #acceptingEvents = false;
  #els;
  #term;
  #terminalWriter;
  #ui;
  #disk;
  #display;
  #getEmulator;
  #saveDisk;
  #handleError;
  #running = false;
  #boundEmulator;
  #bootTimeline;
  #onBootTimeline;
  #parallelTransition;
  #stagedSmp = false;
  #uartTail = "";
  #uartProbeText;

  constructor({
    els,
    term,
    ui,
    disk,
    display,
    getEmulator,
    saveDisk,
    handleError,
    now,
    onBootTimeline,
  }) {
    this.#els = els;
    this.#term = term;
    this.#terminalWriter = new TerminalWriter(term, {
      autoScroll: () => this.#els.autoScroll.checked,
    });
    this.#ui = ui;
    this.#disk = disk;
    this.#display = display;
    this.#getEmulator = getEmulator;
    this.#saveDisk = saveDisk;
    this.#handleError = handleError;
    this.#onBootTimeline = onBootTimeline;
    this.#parallelTransition = new BootParallelTransition({
      disk,
      getEmulator,
      handleError,
      ui,
    });
    this.#bootTimeline = new UartBootTimeline({
      now,
      onMilestone: (milestone) => this.#handleBootMilestone(milestone),
    });
    this.#els.stepSlice.addEventListener("input", () => {
      this.#getEmulator()?.set_step_slice(this.#stepSlice());
    });
    this.#uartProbeText = installUartProbe();
  }

  start({ installedSystem = false, stagedSmp = false } = {}) {
    this.#acceptingEvents = true;
    this.#running = true;
    this.#stagedSmp = stagedSmp;
    this.#beginBoot(installedSystem);
    const emulator = this.#bindCurrentEmulator();
    emulator?.start(this.#stepSlice());
  }

  resume() {
    this.#acceptingEvents = true;
    this.#running = true;
    const emulator = this.#bindCurrentEmulator();
    emulator?.resume(this.#stepSlice());
  }

  pause() {
    this.#running = false;
    this.#getEmulator()?.pause();
  }

  stop() {
    this.#acceptingEvents = false;
    this.#running = false;
    this.#deactivateBootTracking();
    this.#getEmulator()?.stop();
  }
  #bindCurrentEmulator() {
    const emulator = this.#getEmulator();
    if (!emulator || emulator === this.#boundEmulator) {
      return emulator;
    }

    const isCurrent = () => this.#acceptingEvents && emulator === this.#getEmulator();
    bindRunnerEvents(emulator, {
      autosave: () => this.#autosave(emulator),
      current: isCurrent,
      error: (error) => this.#workerError(error),
      frame2d: (packet) => this.#display?.present(packet),
      frame3d: (packet) => this.#display?.present3d(packet),
      gpuReset: () => this.#display?.reset(),
      metrics: () => this.#ui.updateMetrics(emulator, this.#disk),
      network: (status) => this.#ui.log(`Network proxy ${status}`),
      uart: (output) => this.#writeUart(output),
    });
    this.#boundEmulator = emulator;
    return emulator;
  }

  #autosave(emulator) {
    if (this.#disk.shouldAutosave(emulator)) {
      this.#saveDisk({ quiet: true }).catch(this.#handleError);
    }
  }

  #workerError(error) {
    this.#acceptingEvents = false;
    this.#running = false;
    this.#deactivateBootTracking();
    this.#handleError(error);
  }

  #writeUart(output) {
    if (!output) return;
    this.#terminalWriter.write(output);
    this.#recordUart(output);
    this.#bootTimeline.observe(output);
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
    this.#parallelTransition.reset();
    this.#terminalWriter.reset();
    if (this.#uartTail) {
      this.#uartTail = "";
      this.#uartProbeText.data = "";
    }
    this.#bootTimeline.start({ installedSystem });
  }

  #deactivateBootTracking() {
    this.#stagedSmp = false;
    this.#parallelTransition.reset();
    this.#bootTimeline.start();
  }

  #handleBootMilestone(milestone) {
    this.#onBootTimeline?.(milestone);
    if (this.#stagedSmp) this.#parallelTransition.observe(milestone);
  }

  #stepSlice() {
    return clamp(Number(this.#els.stepSlice.value) || DEFAULT_STEP_SLICE, 1000, MAX_STEP_SLICE);
  }
}
