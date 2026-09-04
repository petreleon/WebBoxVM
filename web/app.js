import { VmBooter } from "./js/boot-vm.js?v=20260904-virgl-depth-texture-r1";
import { formatBootMilestone } from "./js/boot-timeline.js?v=20260904-virgl-depth-texture-r1";
import { els } from "./js/dom.js?v=20260904-virgl-depth-texture-r1";
import { DiskPersistence } from "./js/persistence.js?v=20260904-virgl-depth-texture-r1";
import { VmRunner } from "./js/runner.js?v=20260904-virgl-depth-texture-r1";
import {
  extraBootargsFromLocation,
  installedDiskBenchmarkFromLocation,
  stagedSmpRequestedFromLocation,
} from "./js/boot-args.js?v=20260904-virgl-depth-texture-r1";
import { installWebboxVmDevtools } from "./js/devtools.js?v=20260904-virgl-depth-texture-r1";
import {
  fetchBundledDebian,
  fetchInstalledDiskBenchmark,
  readSelectedIso,
} from "./js/sources.js?v=20260904-virgl-depth-texture-r1";
import { mountTerminal, waitForTerminal } from "./js/terminal.js?v=20260904-virgl-depth-texture-r1";
import { UiController } from "./js/ui.js?v=20260904-virgl-depth-texture-r1";
import { GuestDisplay } from "./js/gpu-display.js?v=20260904-virgl-depth-texture-r1";

const ui = new UiController(els);
const display = new GuestDisplay(els.displayCanvas, els.displayStatus);
void display.acquireWebGpuBackend().catch(() => {});
const disk = new DiskPersistence();
const diskBootExtraArgs = extraBootargsFromLocation();
const installedDiskBenchmark = installedDiskBenchmarkFromLocation();
const stagedSmpRequested = stagedSmpRequestedFromLocation();

let term, emulator, runner, booter;
let bootedName = "";

await waitForTerminal();
term = mountTerminal(els, () => emulator);
const devtools = installWebboxVmDevtools(() => emulator, () => runner);
runner = new VmRunner({
  els,
  term,
  ui,
  disk,
  display,
  getEmulator: () => emulator,
  saveDisk,
  handleError,
  onBootTimeline: (milestone) => ui.log(formatBootMilestone(milestone)),
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
  getJitEnabled: () => devtools.jitEnabled(),
  resetEmulator: resetEmulatorOnly,
  onBooted: (name) => {
    bootedName = name;
  },
});
wireEvents();
ui.setControls("idle", disk, emulator);
ui.updateStorageMetric(disk);
const persistenceReady = installedDiskBenchmark
  ? Promise.resolve()
  : disk.init((message) => ui.log(message)).catch(handlePersistenceError);
persistenceReady.then(() => ui.setControls(ui.controlState, disk, emulator));
ui.log("Ready");
if (installedDiskBenchmark) {
  ui.log("Benchmark mode: persistence disabled");
  bootInstalledDiskBenchmark().catch(handleError);
}

function wireEvents() {
  els.bootIso.addEventListener("click", () => bootFrom(readSelectedIso(els, ui)).catch(handleError));
  els.bootDebian.addEventListener("click", () => bootFrom(fetchBundledDebian(ui)).catch(handleError));
  els.bootDisk.addEventListener("click", () => bootDisk().catch(handleError));
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

async function bootDisk() {
  await booter.bootSavedDisk(persistenceReady, diskBootExtraArgs);
}

async function bootInstalledDiskBenchmark() {
  const source = await fetchInstalledDiskBenchmark(ui);
  await booter.bootInstalledSnapshot(
    source.bytes,
    source.name,
    diskBootExtraArgs,
    stagedSmpRequested,
  );
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
  display.reset();
  if (emulator) {
    emulator.free();
    emulator = undefined;
  }
}
