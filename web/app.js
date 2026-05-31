import { VmBooter } from "./js/boot-vm.js";
import { els } from "./js/dom.js";
import { DiskPersistence } from "./js/persistence.js";
import { VmRunner } from "./js/runner.js";
import { fetchBundledDebian, readSelectedIso } from "./js/sources.js";
import { mountTerminal, waitForTerminal } from "./js/terminal.js";
import { UiController } from "./js/ui.js";

const ui = new UiController(els);
const disk = new DiskPersistence();

let term;
let emulator;
let runner;
let booter;
let bootedName = "";

await waitForTerminal();
term = mountTerminal(els, () => emulator);
runner = new VmRunner({
  els,
  term,
  ui,
  disk,
  getEmulator: () => emulator,
  saveDisk,
  handleError,
});
booter = new VmBooter({
  els,
  ui,
  disk,
  term,
  runner,
  setEmulator: (value) => {
    emulator = value;
  },
  resetEmulator: resetEmulatorOnly,
  onBooted: (name) => {
    bootedName = name;
  },
});
wireEvents();
ui.setControls("idle", disk, emulator);
ui.updateStorageMetric(disk);
const persistenceReady = disk.init((message) => ui.log(message)).catch(handlePersistenceError);
ui.log("Ready");

function wireEvents() {
  els.bootIso.addEventListener("click", () => bootFrom(readSelectedIso(els, ui)).catch(handleError));
  els.bootDebian.addEventListener("click", () => bootFrom(fetchBundledDebian(ui)).catch(handleError));
  els.pauseVm.addEventListener("click", () => pauseVm());
  els.resumeVm.addEventListener("click", resumeVm);
  els.resetVm.addEventListener("click", () => resetVm().catch(handleError));
  els.saveDisk.addEventListener("click", () => saveDisk({ force: true }).catch(handleError));
  els.clearDisk.addEventListener("click", () => clearDisk().catch(handleError));
  window.addEventListener("pagehide", () => {
    if (emulator) {
      saveDisk({ force: true, quiet: true }).catch(() => {});
    }
  });
}

function handlePersistenceError(error) {
  disk.available = false;
  ui.updateStorageMetric(disk);
  ui.log(error.stack ?? String(error));
  ui.setControls(ui.controlState, disk, emulator);
}

function handleError(error) {
  runner?.pause();
  ui.setControls(emulator ? "paused" : "idle", disk, emulator);
  ui.setStatus(error.message ?? String(error), "error");
  ui.log(error.stack ?? String(error));
}

async function bootFrom(sourcePromise) {
  const source = await sourcePromise;
  if (!source) {
    return;
  }
  await booter.bootBytes(source.bytes, source.name, persistenceReady);
}

function pauseVm() {
  runner.pause();
  ui.setControls("paused", disk, emulator);
  ui.setStatus(`Paused ${bootedName}`);
  saveDisk({ force: true }).catch(handleError);
}

function resumeVm() {
  if (!emulator) {
    return;
  }
  runner.resume();
  ui.setControls("running", disk, emulator);
  ui.setStatus(`Running ${bootedName}`);
}

async function resetVm() {
  runner.stop();
  ui.setControls("booting", disk, emulator);
  ui.setStatus("Saving disk");
  await saveDisk({ force: true });
  resetEmulatorOnly();
  bootedName = "";
  term.clear();
  ui.updateMetrics(emulator, disk);
  ui.setControls("idle", disk, emulator);
  ui.setStatus("Idle");
  ui.log("Reset");
}

async function saveDisk(options = {}) {
  await disk.save(emulator, {
    ...options,
    after: () => {
      ui.updateStorageMetric(disk);
      ui.setControls(ui.controlState, disk, emulator);
    },
    log: (message) => ui.log(message),
  });
}

async function clearDisk() {
  await disk.clear(emulator, (message) => ui.log(message));
  ui.updateStorageMetric(disk);
  ui.setControls(ui.controlState, disk, emulator);
}

function resetEmulatorOnly() {
  if (emulator) {
    emulator.free();
    emulator = undefined;
  }
}
