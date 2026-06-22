import assert from "node:assert/strict";
import test from "node:test";
import { compiledJitBlockSkipReason } from "./jit-compile.js";

test("EL0 guest-memory helper blocks are not compile-time skipped", () => {
  assert.equal(
    compiledJitBlockSkipReason({ blockEl: 0, usesGuestHelpers: true }),
    undefined,
  );
});
