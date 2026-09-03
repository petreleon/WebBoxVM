import assert from "node:assert/strict";
import test from "node:test";
import { UiController } from "./ui.js?v=20260903-virgl-capset1-r1";

test("active metric updates avoid duplicate jit stats reads", () => {
  let jitStatsCalls = 0;
  const ui = new UiController(elements());

  ui.updateMetrics(
    {
      allocated_pages: () => 1,
      install_disk_allocated_bytes: () => 0,
      jit_stats: () => {
        jitStatsCalls += 1;
        return { cacheBlocks: 1 };
      },
      network_stats: () => ({
        rxPackets: 2n,
        status: "connected",
        txPackets: 3n,
      }),
      pc: () => 0x1000n,
      total_steps: () => 4n,
      uart_output_len: () => 0,
    },
    disk(),
  );

  assert.equal(jitStatsCalls, 0);
});

test("empty metric updates clear hidden jit stats", () => {
  const els = elements();
  els.jitStatsValue.textContent = "{\"cacheBlocks\":1}";
  const ui = new UiController(els);

  ui.updateMetrics(undefined, disk());

  assert.equal(els.jitStatsValue.textContent, "null");
});

test("unchanged active metrics do not rewrite text nodes", () => {
  const els = elements(countedElement);
  const ui = new UiController(els);

  ui.updateMetrics(metricEmulator(), disk());
  const writesAfterFirstUpdate = metricWrites(els);
  assert.ok(writesAfterFirstUpdate > 0);

  ui.updateMetrics(metricEmulator(), disk());

  assert.equal(metricWrites(els), writesAfterFirstUpdate);
});

test("unchanged empty metrics do not rewrite cleared text nodes", () => {
  const els = elements(countedElement);
  const ui = new UiController(els);

  ui.updateMetrics(undefined, disk());
  const writesAfterFirstUpdate = metricWrites(els);

  ui.updateMetrics(undefined, disk());

  assert.equal(metricWrites(els), writesAfterFirstUpdate);
});

test("unchanged status text is not rewritten", () => {
  const els = elements(countedElement);
  const ui = new UiController(els);

  ui.setStatus("Booting disk", "normal");
  ui.setStatus("Booting disk", "normal");
  ui.setStatus("Booting disk", "warn");

  assert.equal(els.statusLine.writeCount, 1);
  assert.equal(els.statusLine.dataset.tone, "warn");
});

function elements(makeElement = basicElement) {
  return {
    bootDebian: makeElement(),
    bootDisk: makeElement(),
    bootIso: makeElement(),
    clearDisk: makeElement(),
    diskSize: makeElement(),
    diskValue: makeElement(),
    eventLog: { scrollHeight: 0, scrollTop: 0, textContent: "" },
    isoFile: makeElement(),
    jitStatsValue: makeElement(),
    netValue: makeElement(),
    pagesValue: makeElement(),
    pauseVm: makeElement(),
    pcValue: makeElement(),
    resetVm: makeElement(),
    resumeVm: makeElement(),
    saveDisk: makeElement(),
    savedValue: makeElement(),
    statusLine: makeElement(),
    stepsValue: makeElement(),
    uartValue: makeElement(),
  };
}

function basicElement() {
  return { dataset: {}, disabled: false, textContent: "" };
}

function countedElement() {
  let textContent = "";
  let writeCount = 0;
  return {
    dataset: {},
    disabled: false,
    get textContent() {
      return textContent;
    },
    set textContent(value) {
      writeCount += 1;
      textContent = value;
    },
    get writeCount() {
      return writeCount;
    },
  };
}

function metricWrites(els) {
  return [
    "diskValue",
    "jitStatsValue",
    "netValue",
    "pagesValue",
    "pcValue",
    "savedValue",
    "stepsValue",
    "uartValue",
  ].reduce((sum, key) => sum + els[key].writeCount, 0);
}

function metricEmulator() {
  return {
    allocated_pages: () => 1,
    install_disk_allocated_bytes: () => 0,
    jit_stats: () => ({ cacheBlocks: 1 }),
    network_stats: () => ({
      rxPackets: 2n,
      status: "connected",
      txPackets: 3n,
    }),
    pc: () => 0x1000n,
    total_steps: () => 4n,
    uart_output_len: () => 0,
  };
}

function disk() {
  return {
    available: true,
    persistedBytes: 0,
    saving: false,
  };
}
