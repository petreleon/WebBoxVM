import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_VM_CORES, jitEnabledForBoot } from "./boot-vm.js";

test("browser boots default to two virtual CPUs", () => {
  assert.equal(DEFAULT_VM_CORES, 2);
});

test("media boots keep jit disabled by default", () => {
  assert.equal(jitEnabledForBoot("media", false), false);
});

test("media boots honor a manual jit enable", () => {
  assert.equal(jitEnabledForBoot("media", true), true);
});

test("saved disk boots enable jit by default", () => {
  assert.equal(jitEnabledForBoot("saved-disk", false), true);
});

test("multicore boots keep the single-core JIT disabled", () => {
  assert.equal(jitEnabledForBoot("media", true, 2), false);
  assert.equal(jitEnabledForBoot("saved-disk", false, 2), false);
});
