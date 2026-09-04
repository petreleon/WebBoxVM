import assert from "node:assert/strict";
import test from "node:test";
import {
  decodeDiskSnapshotFromStorage,
  encodeDiskSnapshotForStorage,
  writeDiskSnapshotToStorage,
} from "./persistence-store.js?v=20260904-virgl-material-batch-r1";

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

test("disk snapshots can stream into storage as compressed chunks", async (t) => {
  if (typeof CompressionStream !== "function" || typeof WritableStream !== "function") {
    t.skip("Web streams compression is unavailable");
  }
  const snapshot = new Uint8Array(3 * 1024 * 1024);
  snapshot.fill(3, 0, 512 * 1024);
  snapshot.fill(9, 2 * 1024 * 1024);
  const chunks = [];
  const writable = new WritableStream({
    write(chunk) {
      chunks.push(Uint8Array.from(chunk));
    },
  });

  await writeDiskSnapshotToStorage(snapshot, writable);
  await writable.close();
  const stored = concat(chunks);
  const restored = await decodeDiskSnapshotFromStorage(stored);

  assert.equal(chunks[0].byteLength, 8);
  assert.ok(stored.byteLength < snapshot.byteLength);
  assert.deepEqual(restored, snapshot);
});

function concat(chunks) {
  const total = chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return out;
}
