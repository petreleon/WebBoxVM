import assert from "node:assert/strict";
import test from "node:test";
import { withEmulatorAccess } from "./access.js?v=20260903-webgpu-virtio-r4";

test("emulator access waits for an active worker round", async () => {
  let releaseRound;
  let markRoundStarted;
  const roundBlocked = new Promise((resolve) => {
    releaseRound = resolve;
  });
  const roundStarted = new Promise((resolve) => {
    markRoundStarted = resolve;
  });
  const order = [];
  const round = withEmulatorAccess(async () => {
    order.push("round-start");
    markRoundStarted();
    await roundBlocked;
    order.push("round-end");
  });
  await roundStarted;
  const input = withEmulatorAccess(() => {
    order.push("input");
  });

  assert.deepEqual(order, ["round-start"]);
  releaseRound();
  await Promise.all([round, input]);

  assert.deepEqual(order, ["round-start", "round-end", "input"]);
});

test("failed access releases the next caller", async () => {
  await assert.rejects(
    withEmulatorAccess(() => {
      throw new Error("worker failed");
    }),
    /worker failed/,
  );
  assert.equal(await withEmulatorAccess(() => 42), 42);
});
