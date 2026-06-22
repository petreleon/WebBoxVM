import assert from "node:assert/strict";
import test from "node:test";
import { jitEnabledForBoot } from "./boot-vm.js";

test("media boots keep jit disabled by default", () => {
  assert.equal(jitEnabledForBoot("media", false), false);
});

test("media boots honor a manual jit enable", () => {
  assert.equal(jitEnabledForBoot("media", true), true);
});

test("saved disk boots enable jit by default", () => {
  assert.equal(jitEnabledForBoot("saved-disk", false), true);
});
