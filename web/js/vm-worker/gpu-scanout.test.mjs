import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { maybePostGpuScanout } from "./gpu-scanout.js?v=20260904-virgl-depth-compare-r1";
import { state } from "./state.js?v=20260904-virgl-depth-compare-r1";

let previousPollAt;
let previousResetGeneration;

beforeEach(() => {
  previousPollAt = state.lastGpuScanoutPollAt;
  previousResetGeneration = state.gpuResetGeneration;
  state.lastGpuScanoutPollAt = Number.NEGATIVE_INFINITY;
  state.gpuResetGeneration = undefined;
});

afterEach(() => {
  state.lastGpuScanoutPollAt = previousPollAt;
  state.gpuResetGeneration = previousResetGeneration;
});

test("scanout polling runs near 60 Hz and posts only nonempty transferable packets", () => {
  const packet = new Uint8Array([1, 2, 3, 4]);
  const updates = [new Uint8Array(), packet];
  const calls = [];
  let polls = 0;
  const emulator = {
    gpu_scanout_update() {
      polls += 1;
      return updates.shift();
    },
  };
  const post = (...args) => calls.push(args);

  assert.equal(maybePostGpuScanout(0, emulator, post), false);
  assert.equal(maybePostGpuScanout(10, emulator, post), false);
  assert.equal(maybePostGpuScanout(17, emulator, post), true);

  assert.equal(polls, 2);
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0][0], { event: "gpuFrame", packet });
  assert.deepEqual(calls[0][1], [packet.buffer]);
});

test("scanout polling copies non-owning views before transfer", () => {
  const backing = new Uint8Array([9, 1, 2, 3, 8]);
  const messages = [];

  assert.equal(
    maybePostGpuScanout(
      0,
      { gpu_scanout_update: () => backing.subarray(1, 4) },
      (message, transfer) => messages.push({ message, transfer }),
    ),
    true,
  );

  assert.deepEqual([...messages[0].message.packet], [1, 2, 3]);
  assert.notEqual(messages[0].message.packet.buffer, backing.buffer);
  assert.deepEqual(messages[0].transfer, [messages[0].message.packet.buffer]);
});

test("scanout polling stays compatible with wasm packages that predate the optional method", () => {
  let posted = false;
  assert.equal(maybePostGpuScanout(0, {}, () => (posted = true)), false);
  assert.equal(posted, false);
});

test("guest GPU reset is posted before a new frame from the reset device", () => {
  let generation = 3;
  const updates = [new Uint8Array(), new Uint8Array([9])];
  const messages = [];
  const emulator = {
    gpu_reset_generation: () => generation,
    gpu_scanout_update: () => updates.shift(),
  };

  assert.equal(maybePostGpuScanout(0, emulator, (message) => messages.push(message)), false);
  generation = 4;
  assert.equal(maybePostGpuScanout(17, emulator, (message) => messages.push(message)), true);
  assert.deepEqual(messages.map(({ event }) => event), ["gpuReset", "gpuFrame"]);
  assert.deepEqual(messages[0], { event: "gpuReset", generation: 4 });
});
