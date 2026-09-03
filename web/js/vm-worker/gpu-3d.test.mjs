import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { maybePostGpu3d } from "./gpu-3d.js?v=20260903-virgl-capset1-r1";
import { state } from "./state.js?v=20260903-virgl-capset1-r1";

let previousPollAt;
beforeEach(() => {
  previousPollAt = state.lastGpu3dPollAt;
  state.lastGpu3dPollAt = Number.NEGATIVE_INFINITY;
});
afterEach(() => { state.lastGpu3dPollAt = previousPollAt; });

test("3D polling runs near 60 Hz and posts only nonempty transferable packets", () => {
  const packet = new Uint8Array([1, 2, 3, 4]);
  const updates = [new Uint8Array(), packet];
  const calls = [];
  let polls = 0;
  const emulator = { gpu_3d_update() { polls += 1; return updates.shift(); } };
  const post = (...args) => calls.push(args);
  assert.equal(maybePostGpu3d(0, emulator, post), false);
  assert.equal(maybePostGpu3d(10, emulator, post), false);
  assert.equal(maybePostGpu3d(17, emulator, post), true);
  assert.equal(polls, 2);
  assert.deepEqual(calls, [[{ event: "gpu3dFrame", packet }, [packet.buffer]]]);
});

test("3D polling copies a non-owning view and tolerates an older wasm package", () => {
  const backing = new Uint8Array([9, 1, 2, 3, 8]);
  let sent;
  assert.equal(maybePostGpu3d(0, { gpu_3d_update: () => backing.subarray(1, 4) }, (message) => {
    sent = message.packet;
  }), true);
  assert.deepEqual([...sent], [1, 2, 3]);
  assert.notEqual(sent.buffer, backing.buffer);
  state.lastGpu3dPollAt = Number.NEGATIVE_INFINITY;
  assert.equal(maybePostGpu3d(0, {}, () => assert.fail("unexpected post")), false);
});
