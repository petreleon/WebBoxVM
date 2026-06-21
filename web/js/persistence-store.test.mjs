import assert from "node:assert/strict";
import test from "node:test";
import {
  decodeDiskSnapshotFromStorage,
  encodeDiskSnapshotForStorage,
} from "./persistence-store.js";

test("compressed disk snapshots roundtrip from storage", async () => {
  const snapshot = new Uint8Array(1024 * 1024);
  snapshot.fill(7, 0, 512 * 1024);

  const stored = await encodeDiskSnapshotForStorage(snapshot);
  const restored = await decodeDiskSnapshotFromStorage(stored);

  assert.ok(stored.byteLength < snapshot.byteLength / 4);
  assert.deepEqual(restored, snapshot);
});

test("legacy raw disk snapshots still load", async () => {
  const snapshot = new Uint8Array([1, 2, 3, 4]);

  const restored = await decodeDiskSnapshotFromStorage(snapshot);

  assert.deepEqual(restored, snapshot);
});
