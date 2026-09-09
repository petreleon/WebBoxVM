import assert from "node:assert/strict";
import test from "node:test";
import { VirglRouteDispatcher } from "./webgpu-3d-dispatch.js?v=20260904-virgl-readback-pool-r1";
import { LegacyWbg3Renderer } from "./webgpu-3d-legacy.js?v=20260904-virgl-readback-pool-r1";
import { ExperimentalWebGpu3dRenderer } from "./webgpu-3d.js?v=20260904-virgl-readback-pool-r1";

test("guest 3D facade keeps VirGL routing separate from WBG3 lifecycle", () => {
  assert.equal(typeof ExperimentalWebGpu3dRenderer, "function");
  assert.equal(typeof VirglRouteDispatcher, "function");
  assert.equal(typeof LegacyWbg3Renderer, "function");
});
