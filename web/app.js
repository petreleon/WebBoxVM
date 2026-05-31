import init, { Emulator } from "./pkg/emulator.js";

const DISK_FILE_NAME = "webboxvm-install-disk.wbdisk";
const AUTOSAVE_INTERVAL_MS = 5000;
const GIB = 1024n ** 3n;

const els = {
  autoScroll: document.querySelector("#autoScroll"),
  bootDebian: document.querySelector("#bootDebian"),
  bootIso: document.querySelector("#bootIso"),
  clearDisk: document.querySelector("#clearDisk"),
  diskSize: document.querySelector("#diskSize"),
  diskValue: document.querySelector("#diskValue"),
  eventLog: document.querySelector("#eventLog"),
  isoFile: document.querySelector("#isoFile"),
  pagesValue: document.querySelector("#pagesValue"),
  pauseVm: document.querySelector("#pauseVm"),
  pcValue: document.querySelector("#pcValue"),
  resetVm: document.querySelector("#resetVm"),
  resumeVm: document.querySelector("#resumeVm"),
  saveDisk: document.querySelector("#saveDisk"),
  savedValue: document.querySelector("#savedValue"),
  statusLine: document.querySelector("#statusLine"),
  stepSlice: document.querySelector("#stepSlice"),
  stepsValue: document.querySelector("#stepsValue"),
  terminal: document.querySelector("#terminal"),
  uartValue: document.querySelector("#uartValue"),
};

let term;
let fitAddon;
let emulator;
let wasmReady = false;
let running = false;
let pumpScheduled = false;
let lastUart = 0;
let bootedName = "";
let controlState = "idle";
let opfsAvailable = false;
let storageRoot;
let lastSavedDiskGeneration = 0n;
let lastPersistedBytes = 0;
let saveInProgress = false;
let saveQueued = false;
let lastAutosaveAt = 0;
let persistenceReady;

await waitForTerminal();
mountTerminal();
wireEvents();
setControls("idle");
updateStorageMetric();
persistenceReady = initPersistence().catch((error) => {
  opfsAvailable = false;
  updateStorageMetric();
  log(error.stack ?? String(error));
  setControls(controlState);
});
log("Ready");

async function waitForTerminal() {
  while (!window.Terminal || !window.FitAddon) {
    await delay(16);
  }
}

function mountTerminal() {
  term = new window.Terminal({
    cursorBlink: true,
    convertEol: true,
    fontFamily: "SFMono-Regular, Menlo, Consolas, monospace",
    fontSize: 13,
    letterSpacing: 0,
    scrollback: 8000,
    theme: {
      background: "#050606",
      foreground: "#edf1ee",
      cursor: "#5cc8a7",
      selectionBackground: "#365950",
      black: "#0b0d0d",
      red: "#df6b62",
      green: "#5cc8a7",
      yellow: "#e4b75a",
      blue: "#79a8d8",
      magenta: "#c98bd6",
      cyan: "#70c6d1",
      white: "#edf1ee",
      brightBlack: "#636d68",
      brightRed: "#f08379",
      brightGreen: "#77d9bb",
      brightYellow: "#f1c86b",
      brightBlue: "#95bee8",
      brightMagenta: "#d8a0e3",
      brightCyan: "#8bd9e2",
      brightWhite: "#ffffff",
    },
  });
  fitAddon = new window.FitAddon.FitAddon();
  term.loadAddon(fitAddon);
  term.open(els.terminal);
  fitTerminal();
  term.onData((data) => {
    if (emulator) {
      emulator.send_uart_input(data);
    }
  });
  window.addEventListener("resize", fitTerminal);
}

function wireEvents() {
  els.bootIso.addEventListener("click", () => bootSelectedIso().catch(handleError));
  els.bootDebian.addEventListener("click", () => bootBundledDebian().catch(handleError));
  els.pauseVm.addEventListener("click", () => {
    running = false;
    setControls("paused");
    setStatus(`Paused ${bootedName}`);
    savePersistentDisk({ force: true }).catch(handleError);
  });
  els.resumeVm.addEventListener("click", () => {
    if (!emulator) {
      return;
    }
    running = true;
    setControls("running");
    setStatus(`Running ${bootedName}`);
    schedulePump();
  });
  els.resetVm.addEventListener("click", () => resetVm().catch(handleError));
  els.saveDisk.addEventListener("click", () => {
    savePersistentDisk({ force: true }).catch(handleError);
  });
  els.clearDisk.addEventListener("click", () => {
    clearPersistentDisk().catch(handleError);
  });
  window.addEventListener("pagehide", () => {
    if (emulator) {
      savePersistentDisk({ force: true, quiet: true }).catch(() => {});
    }
  });
}

function handleError(error) {
  running = false;
  setControls(emulator ? "paused" : "idle");
  setStatus(error.message ?? String(error), "error");
  log(error.stack ?? String(error));
}

async function bootSelectedIso() {
  const file = els.isoFile.files?.[0];
  if (!file) {
    setStatus("No ISO selected", "warn");
    log("No ISO selected");
    return;
  }

  setStatus(`Reading ${file.name}`);
  log(`Reading ${file.name} (${formatBytes(file.size)})`);
  await nextFrame();

  let buffer = await file.arrayBuffer();
  let bytes = new Uint8Array(buffer);
  await bootBytes(bytes, file.name);
  bytes = null;
  buffer = null;
}

async function bootBundledDebian() {
  const url = "./media/debian-arm64-netinst.iso";
  setStatus("Fetching Debian ISO");
  log(`Fetching ${url}`);
  await nextFrame();

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Debian ISO fetch failed: HTTP ${response.status}`);
  }
  let buffer = await response.arrayBuffer();
  let bytes = new Uint8Array(buffer);
  await bootBytes(bytes, "Debian arm64 netinst");
  bytes = null;
  buffer = null;
}

async function bootBytes(bytes, name) {
  try {
    await ensureWasm();
    resetEmulatorOnly();
    term.clear();
    term.write(`Booting ${name}\r\n`);
    setControls("booting");
    setStatus(`Booting ${name}`);
    await nextFrame();

    emulator = new Emulator(1);
    const diskSizeBytes = BigInt(clamp(Number(els.diskSize.value) || 4, 1, 64)) * GIB;
    const result = emulator.boot_iso_with_disk(bytes, 1, diskSizeBytes);
    log(result);
    if (result.startsWith("ERR:")) {
      setStatus(result, "error");
      setControls("idle");
      return;
    }

    await persistenceReady;
    const restoreMessage = await restorePersistentDiskIfPresent();
    if (restoreMessage) {
      log(restoreMessage);
      syncDiskSizeInputFromEmulator();
    }
    lastSavedDiskGeneration = emulator.install_disk_generation();
    bootedName = name;
    lastUart = 0;
    running = true;
    setControls("running");
    setStatus(`Running ${name}`);
    term.focus();
    updateMetrics();
    schedulePump();
  } catch (error) {
    setStatus(error.message ?? String(error), "error");
    log(error.stack ?? String(error));
    setControls(emulator ? "paused" : "idle");
  }
}

async function ensureWasm() {
  if (wasmReady) {
    return;
  }
  setStatus("Loading WASM");
  await init(new URL("./pkg/emulator_bg.wasm", import.meta.url));
  wasmReady = true;
  log("WASM loaded");
}

async function initPersistence() {
  opfsAvailable = Boolean(navigator.storage?.getDirectory);
  if (!opfsAvailable) {
    updateStorageMetric();
    log("Persistent disk storage unavailable");
    setControls(controlState);
    return;
  }

  try {
    if (navigator.storage.persist) {
      await navigator.storage.persist();
    }
    await refreshPersistentDiskInfo();
    log(
      lastPersistedBytes > 0
        ? `Persistent disk ready (${formatBytes(lastPersistedBytes)})`
        : "Persistent disk ready",
    );
  } catch (error) {
    opfsAvailable = false;
    throw error;
  } finally {
    updateStorageMetric();
    setControls(controlState);
  }
}

async function getStorageRoot() {
  if (!opfsAvailable) {
    return undefined;
  }
  if (!storageRoot) {
    storageRoot = await navigator.storage.getDirectory();
  }
  return storageRoot;
}

async function refreshPersistentDiskInfo() {
  const root = await getStorageRoot();
  if (!root) {
    return;
  }

  try {
    const handle = await root.getFileHandle(DISK_FILE_NAME);
    const file = await handle.getFile();
    lastPersistedBytes = file.size;
  } catch (error) {
    if (error.name !== "NotFoundError") {
      throw error;
    }
    lastPersistedBytes = 0;
  }
  updateStorageMetric();
}

async function loadPersistentDisk() {
  const root = await getStorageRoot();
  if (!root) {
    return undefined;
  }

  try {
    const handle = await root.getFileHandle(DISK_FILE_NAME);
    const file = await handle.getFile();
    if (file.size === 0) {
      lastPersistedBytes = 0;
      updateStorageMetric();
      return undefined;
    }
    const bytes = new Uint8Array(await file.arrayBuffer());
    lastPersistedBytes = bytes.byteLength;
    updateStorageMetric();
    return bytes;
  } catch (error) {
    if (error.name !== "NotFoundError") {
      throw error;
    }
    lastPersistedBytes = 0;
    updateStorageMetric();
    return undefined;
  }
}

async function restorePersistentDiskIfPresent() {
  const snapshot = await loadPersistentDisk();
  if (!snapshot) {
    return "";
  }

  const result = emulator.restore_install_disk(snapshot);
  if (result.startsWith("ERR:")) {
    throw new Error(result);
  }
  return `${result} from ${formatBytes(snapshot.byteLength)} OPFS snapshot`;
}

async function writePersistentDisk(snapshot) {
  const root = await getStorageRoot();
  if (!root) {
    return;
  }

  const handle = await root.getFileHandle(DISK_FILE_NAME, { create: true });
  const writable = await handle.createWritable();
  await writable.write(snapshot);
  await writable.close();
  lastPersistedBytes = snapshot.byteLength;
  updateStorageMetric();
}

async function savePersistentDisk({ force = false, quiet = false } = {}) {
  if (!emulator || !opfsAvailable) {
    return;
  }

  const generation = emulator.install_disk_generation();
  if (!force && generation === lastSavedDiskGeneration) {
    return;
  }
  if (saveInProgress) {
    saveQueued = true;
    return;
  }

  saveInProgress = true;
  updateStorageMetric();
  setControls(controlState);
  const savedGeneration = generation;

  try {
    const snapshot = emulator.install_disk_snapshot();
    await writePersistentDisk(snapshot);
    lastSavedDiskGeneration = savedGeneration;
    if (!quiet) {
      log(`Saved disk (${formatBytes(snapshot.byteLength)})`);
    }
  } finally {
    saveInProgress = false;
    updateStorageMetric();
    setControls(controlState);
    if (saveQueued) {
      saveQueued = false;
      savePersistentDisk({ quiet: true }).catch(handleError);
    }
  }
}

async function clearPersistentDisk() {
  const root = await getStorageRoot();
  if (!root) {
    return;
  }

  try {
    await root.removeEntry(DISK_FILE_NAME);
  } catch (error) {
    if (error.name !== "NotFoundError") {
      throw error;
    }
  }
  lastPersistedBytes = 0;
  lastSavedDiskGeneration = emulator ? emulator.install_disk_generation() : 0n;
  updateStorageMetric();
  setControls(controlState);
  log("Cleared saved disk");
}

function scheduleDiskAutosave() {
  if (!emulator || !opfsAvailable) {
    return;
  }

  const generation = emulator.install_disk_generation();
  if (generation === lastSavedDiskGeneration) {
    return;
  }
  const now = performance.now();
  if (now - lastAutosaveAt < AUTOSAVE_INTERVAL_MS) {
    return;
  }
  lastAutosaveAt = now;
  savePersistentDisk({ quiet: true }).catch(handleError);
}

function schedulePump() {
  if (!running || pumpScheduled) {
    return;
  }
  pumpScheduled = true;
  requestAnimationFrame(runFrame);
}

function runFrame() {
  pumpScheduled = false;
  if (!running || !emulator) {
    return;
  }

  const frameStart = performance.now();
  const stepSlice = clamp(Number(els.stepSlice.value) || 50000, 1000, 1000000);
  let batches = 0;

  try {
    do {
      emulator.run_kernel(stepSlice);
      drainUart();
      batches += 1;
    } while (running && performance.now() - frameStart < 24 && batches < 8);

    updateMetrics();
    scheduleDiskAutosave();
    schedulePump();
  } catch (error) {
    running = false;
    setControls("paused");
    setStatus(error.message ?? String(error), "error");
    log(error.stack ?? String(error));
  }
}

function drainUart() {
  const output = emulator.uart_output_since(lastUart);
  if (!output) {
    return;
  }
  lastUart = emulator.uart_output_len();
  term.write(output);
  if (els.autoScroll.checked) {
    term.scrollToBottom();
  }
}

async function resetVm() {
  running = false;
  setControls("booting");
  setStatus("Saving disk");
  await savePersistentDisk({ force: true });
  resetEmulatorOnly();
  lastUart = 0;
  bootedName = "";
  term.clear();
  updateMetrics();
  setControls("idle");
  setStatus("Idle");
  log("Reset");
}

function resetEmulatorOnly() {
  if (emulator) {
    emulator.free();
    emulator = undefined;
  }
}

function setControls(state) {
  controlState = state;
  const busy = state === "booting";
  const active = state === "running";
  const paused = state === "paused";
  const hasVm = Boolean(emulator);

  els.bootIso.disabled = busy || active;
  els.bootDebian.disabled = busy || active;
  els.diskSize.disabled = busy || active;
  els.isoFile.disabled = busy || active;
  els.pauseVm.disabled = !active;
  els.resumeVm.disabled = !paused;
  els.resetVm.disabled = !(active || paused || busy);
  els.saveDisk.disabled = busy || !opfsAvailable || !hasVm || saveInProgress;
  els.clearDisk.disabled = busy || active || !opfsAvailable || lastPersistedBytes === 0;
}

function updateMetrics() {
  if (!emulator) {
    els.stepsValue.textContent = "0";
    els.pcValue.textContent = "0x0";
    els.uartValue.textContent = "0 B";
    els.pagesValue.textContent = "0";
    els.diskValue.textContent = "0 B";
    updateStorageMetric();
    return;
  }

  els.stepsValue.textContent = emulator.total_steps().toString();
  els.pcValue.textContent = `0x${emulator.pc().toString(16)}`;
  els.uartValue.textContent = formatBytes(emulator.uart_output_len());
  els.pagesValue.textContent = emulator.allocated_pages().toString();
  els.diskValue.textContent = formatBytes(Number(emulator.install_disk_allocated_bytes()));
  updateStorageMetric();
}

function updateStorageMetric() {
  if (!opfsAvailable) {
    els.savedValue.textContent = "Off";
  } else if (saveInProgress) {
    els.savedValue.textContent = "Saving";
  } else if (lastPersistedBytes > 0) {
    els.savedValue.textContent = formatBytes(lastPersistedBytes);
  } else {
    els.savedValue.textContent = "Ready";
  }
}

function setStatus(message, tone = "normal") {
  els.statusLine.textContent = message;
  els.statusLine.dataset.tone = tone;
}

function log(message) {
  const timestamp = new Date().toLocaleTimeString();
  els.eventLog.textContent += `[${timestamp}] ${message}\n`;
  const lines = els.eventLog.textContent.split("\n");
  if (lines.length > 200) {
    els.eventLog.textContent = `${lines.slice(-200).join("\n")}\n`;
  }
  els.eventLog.scrollTop = els.eventLog.scrollHeight;
}

function fitTerminal() {
  requestAnimationFrame(() => {
    try {
      fitAddon.fit();
    } catch {
      // The terminal can be measured only after layout has settled.
    }
  });
}

function syncDiskSizeInputFromEmulator() {
  if (!emulator) {
    return;
  }
  const sizeBytes = emulator.install_disk_size_bytes();
  const gib = Number((sizeBytes + GIB - 1n) / GIB);
  els.diskSize.value = String(clamp(gib, 1, 64));
}

function formatBytes(bytes) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KiB", "MiB", "GiB"];
  let value = bytes / 1024;
  let unit = units.shift();
  while (value >= 1024 && units.length > 0) {
    value /= 1024;
    unit = units.shift();
  }
  return `${value.toFixed(value >= 10 ? 1 : 2)} ${unit}`;
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function nextFrame() {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}
