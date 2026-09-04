import assert from "node:assert/strict";
import test from "node:test";
import { fakeDevice } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { VirglResidentOutputTargets } from "./webgpu-virgl-output-target.js?v=20260904-virgl-readback-pool-r1";

function backend(device) { return { device, deviceGeneration: 1, format: "bgra8unorm" }; }
function frame(sequence, width, height) { return { canvasHeight: height, canvasWidth: width, residentCandidate: true, sequence }; }

test("VirGL resident output targets share a fixed total byte budget", () => {
  const targets = new VirglResidentOutputTargets(); const device = fakeDevice(); const gpu = backend(device);
  for (let sequence = 1; sequence <= 5; sequence += 1) {
    const output = targets.acquire(gpu, frame(sequence, 1024, 768));
    assert.ok(output); assert.equal(targets.publish(gpu, output), true);
  }
  assert.equal(targets.acquire(gpu, frame(6, 1024, 768)), undefined);
  targets.release(1);
  assert.ok(targets.acquire(gpu, frame(6, 1024, 768)));
});

test("VirGL resident output targets admit bounded small-target fanout", () => {
  const targets = new VirglResidentOutputTargets(); const device = fakeDevice(); const gpu = backend(device);
  for (let sequence = 1; sequence <= 16; sequence += 1) {
    const output = targets.acquire(gpu, frame(sequence, 256, 256));
    assert.ok(output); assert.equal(targets.publish(gpu, output), true);
  }
  assert.equal(targets.acquire(gpu, frame(17, 256, 256)), undefined);
});

test("VirGL resident output invalidation releases unpublished reservations", () => {
  const targets = new VirglResidentOutputTargets(); const device = fakeDevice(); const gpu = backend(device);
  for (let sequence = 1; sequence <= 5; sequence += 1) assert.ok(targets.acquire(gpu, frame(sequence, 1024, 768)));
  targets.invalidate();
  assert.ok(targets.acquire(gpu, frame(6, 1024, 768)));
  assert.equal(device.textures.slice(0, 5).every((texture) => texture.destroyed), true);
});
