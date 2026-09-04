import assert from "node:assert/strict";
import test from "node:test";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-depth-texture-color-r1";

test("WebGPU scopes close before asynchronous queue work can interleave", async () => {
  const events = [];
  const stack = [];
  let finish;
  const work = new Promise((resolve) => { finish = resolve; });
  const device = {
    popErrorScope() {
      events.push(`pop:${stack.pop()}`);
      return Promise.resolve(null);
    },
    pushErrorScope(filter) {
      stack.push(filter);
      events.push(`push:${filter}`);
    },
  };

  const captured = captureWebGpuErrors(device, () => {
    events.push("issue");
    return work;
  });
  assert.deepEqual(events, [
    "push:out-of-memory",
    "push:validation",
    "issue",
    "pop:validation",
    "pop:out-of-memory",
  ]);
  finish("complete");
  assert.equal(await captured, "complete");
});
