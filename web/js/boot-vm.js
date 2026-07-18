import { GIB, clamp, nextFrame } from "./utils.js";
import { assertWasm64Supported } from "./wasm64.js";
import { WorkerVm } from "./worker-vm.js";
import { formatBootPhase } from "./boot-timeline.js";

const BOOT_KIND_MEDIA = "media";
const BOOT_KIND_SAVED_DISK = "saved-disk";
export const DEFAULT_VM_CORES = 2;

export class VmBooter {
  #els;
  #ui;
  #disk;
  #term;
  #runner;
  #setEmulator;
  #getJitEnabled;
  #now;
  #resetEmulator;
  #onBooted;
  #wasmReady = false;

  constructor({
    els,
    ui,
    disk,
    term,
    runner,
    setEmulator,
    getJitEnabled,
    resetEmulator,
    onBooted,
    now = () => performance.now(),
  }) {
    this.#els = els;
    this.#ui = ui;
    this.#disk = disk;
    this.#term = term;
    this.#runner = runner;
    this.#setEmulator = setEmulator;
    this.#getJitEnabled = getJitEnabled;
    this.#now = now;
    this.#resetEmulator = resetEmulator;
    this.#onBooted = onBooted;
  }

  async bootBytes(bytes, name, persistenceReady) {
    await this.#ensureWasm();
    this.#resetEmulator();
    this.#term.clear();
    this.#term.write(`Booting ${name}\r\n`);
    this.#ui.setControls("booting", this.#disk, undefined);
    this.#ui.setStatus(`Booting ${name}`);
    await nextFrame();

    const emulator = new WorkerVm();
    emulator.set_jit_enabled(
      jitEnabledForBoot(BOOT_KIND_MEDIA, this.#getJitEnabled(), DEFAULT_VM_CORES),
    );
    this.#setEmulator(emulator);
    const result = await emulator.boot_iso_with_disk(bytes, DEFAULT_VM_CORES, this.#diskSizeBytes());
    this.#ui.log(result);
    if (result.startsWith("ERR:")) {
      this.#ui.setStatus(result, "error");
      this.#ui.setControls("idle", this.#disk, emulator);
      return;
    }

    await this.#restoreDisk(emulator, persistenceReady);
    this.#onBooted(name);
    this.#runner.start();
    this.#ui.setControls("running", this.#disk, emulator);
    this.#ui.setStatus(`Running ${name}`);
    this.#term.focus();
    this.#ui.updateMetrics(emulator, this.#disk);
  }

  async bootSavedDisk(persistenceReady, extraBootargs = "") {
    await this.#ensureWasm();
    await persistenceReady;
    const loadStartedAt = this.#now();
    const snapshot = await this.#disk.load();
    this.#ui.log(formatBootPhase("OPFS load", this.#now() - loadStartedAt));
    if (!snapshot) {
      throw new Error("No saved disk snapshot is available");
    }

    const name = "saved disk";
    this.#resetEmulator();
    this.#term.clear();
    this.#term.write(`Booting ${name}\r\n`);
    this.#ui.setControls("booting", this.#disk, undefined);
    this.#ui.setStatus(`Booting ${name}`);
    await nextFrame();

    const emulator = new WorkerVm();
    emulator.set_jit_enabled(
      jitEnabledForBoot(BOOT_KIND_SAVED_DISK, this.#getJitEnabled(), DEFAULT_VM_CORES),
    );
    this.#setEmulator(emulator);
    const result = await emulator.boot_installed_disk(snapshot, DEFAULT_VM_CORES, extraBootargs);
    this.#ui.log(result);
    this.#logWorkerBootTimings(emulator);
    if (result.startsWith("ERR:")) {
      this.#ui.setStatus(result, "error");
      this.#ui.setControls("idle", this.#disk, emulator);
      return;
    }

    this.#disk.markClean(emulator);
    this.#syncDiskSizeInput(emulator);
    this.#onBooted(name);
    this.#runner.start({ installedSystem: true });
    this.#ui.setControls("running", this.#disk, emulator);
    this.#ui.setStatus(`Running ${name}`);
    this.#term.focus();
    this.#ui.updateMetrics(emulator, this.#disk);
  }

  async #ensureWasm() {
    if (this.#wasmReady) {
      return;
    }
    this.#ui.setStatus("Loading Wasm64");
    assertWasm64Supported();
    this.#wasmReady = true;
    this.#ui.log("Wasm64 runtime supported");
  }

  async #restoreDisk(emulator, persistenceReady) {
    await persistenceReady;
    const restoreMessage = await this.#disk.restoreIfPresent(emulator);
    if (restoreMessage) {
      this.#ui.log(restoreMessage);
      this.#syncDiskSizeInput(emulator);
    }
    this.#disk.markClean(emulator);
  }

  #logWorkerBootTimings(emulator) {
    const timings = emulator.boot_timings?.();
    if (!timings) {
      return;
    }
    this.#ui.log(formatBootPhase("firmware preparation", timings.firmwarePreparationMs));
    this.#ui.log(formatBootPhase("worker pool", timings.workerPoolMs));
  }

  #diskSizeBytes() {
    const gib = clamp(Number(this.#els.diskSize.value) || 4, 1, 64);
    return BigInt(gib) * GIB;
  }

  #syncDiskSizeInput(emulator) {
    const sizeBytes = emulator.install_disk_size_bytes();
    const gib = Number((sizeBytes + GIB - 1n) / GIB);
    this.#els.diskSize.value = String(clamp(gib, 1, 64));
  }
}

export function jitEnabledForBoot(bootKind, manualEnabled, numCores = 1) {
  if (numCores !== 1) {
    return false;
  }
  return bootKind === BOOT_KIND_SAVED_DISK || Boolean(manualEnabled);
}
