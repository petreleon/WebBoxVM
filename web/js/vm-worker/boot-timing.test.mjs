import assert from "node:assert/strict";
import test from "node:test";
import { BootPhaseTimer } from "./boot-timing.js?v=20260720-input-latency-r4";

test("boot phase timer records adjacent phases and overall time", () => {
  const samples = [100, 105, 117.5, 125];
  const timer = new BootPhaseTimer(() => samples.shift());

  timer.end("firmwarePreparationMs");
  timer.end("workerPoolMs");

  assert.deepEqual(timer.finish(), {
    firmwarePreparationMs: 5,
    totalMs: 25,
    workerPoolMs: 12.5,
  });
});
