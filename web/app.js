import init, { Emulator } from "./pkg/emulator.js";

const els = {
  autoScroll: document.querySelector("#autoScroll"),
  bootDebian: document.querySelector("#bootDebian"),
  bootIso: document.querySelector("#bootIso"),
  eventLog: document.querySelector("#eventLog"),
  isoFile: document.querySelector("#isoFile"),
  pagesValue: document.querySelector("#pagesValue"),
  pauseVm: document.querySelector("#pauseVm"),
  pcValue: document.querySelector("#pcValue"),
  resetVm: document.querySelector("#resetVm"),
  resumeVm: document.querySelector("#resumeVm"),
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

await waitForTerminal();
mountTerminal();
wireEvents();
setControls("idle");
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
  els.resetVm.addEventListener("click", resetVm);
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
    const result = emulator.boot_iso(bytes, 1);
    log(result);
    if (result.startsWith("ERR:")) {
      setStatus(result, "error");
      setControls("idle");
      return;
    }

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

function resetVm() {
  running = false;
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
  const busy = state === "booting";
  const active = state === "running";
  const paused = state === "paused";

  els.bootIso.disabled = busy || active;
  els.bootDebian.disabled = busy || active;
  els.pauseVm.disabled = !active;
  els.resumeVm.disabled = !paused;
  els.resetVm.disabled = !(active || paused || busy);
}

function updateMetrics() {
  if (!emulator) {
    els.stepsValue.textContent = "0";
    els.pcValue.textContent = "0x0";
    els.uartValue.textContent = "0 B";
    els.pagesValue.textContent = "0";
    return;
  }

  els.stepsValue.textContent = emulator.total_steps().toString();
  els.pcValue.textContent = `0x${emulator.pc().toString(16)}`;
  els.uartValue.textContent = formatBytes(emulator.uart_output_len());
  els.pagesValue.textContent = emulator.allocated_pages().toString();
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
