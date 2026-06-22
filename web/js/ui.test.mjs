import assert from "node:assert/strict";
import test from "node:test";
import { UiController } from "./ui.js";

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

function elements() {
  const element = () => ({ dataset: {}, disabled: false, textContent: "" });
  return {
    bootDebian: element(),
    bootDisk: element(),
    bootIso: element(),
    clearDisk: element(),
    diskSize: element(),
    diskValue: element(),
    eventLog: { scrollHeight: 0, scrollTop: 0, textContent: "" },
    isoFile: element(),
    jitStatsValue: element(),
    netValue: element(),
    pagesValue: element(),
    pauseVm: element(),
    pcValue: element(),
    resetVm: element(),
    resumeVm: element(),
    saveDisk: element(),
    savedValue: element(),
    statusLine: element(),
    stepsValue: element(),
    uartValue: element(),
  };
}

function disk() {
  return {
    available: true,
    persistedBytes: 0,
    saving: false,
  };
}
