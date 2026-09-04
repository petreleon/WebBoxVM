import assert from "node:assert/strict";
import test, { after } from "node:test";
import { fetchInstalledDiskBenchmark } from "./sources.js?v=20260904-virgl-material-batch-r1";

const previousAnimationFrame = globalThis.requestAnimationFrame;
globalThis.requestAnimationFrame = (callback) => callback();
after(() => {
  if (previousAnimationFrame === undefined) {
    delete globalThis.requestAnimationFrame;
  } else {
    globalThis.requestAnimationFrame = previousAnimationFrame;
  }
});

test("benchmark disk loader fetches the fixed same-origin WBDISK", async () => {
  const snapshot = validSnapshot();
  const calls = [];
  const logs = [];

  const source = await fetchInstalledDiskBenchmark(
    {
      log: (message) => logs.push(message),
      setStatus: (message) => logs.push(message),
    },
    {
      expectedBytes: snapshot.byteLength,
      fetchImpl: async (...args) => {
        calls.push(args);
        return fakeResponse(snapshot);
      },
      locationLike: {
        href: "http://localhost:8787/web/index.html?benchmark=installed-disk",
      },
    },
  );

  assert.equal(calls[0][0], "http://localhost:8787/web/media/benchmark-installed.wbdisk");
  assert.deepEqual(calls[0][1], {
    cache: "no-store",
    credentials: "same-origin",
    mode: "same-origin",
  });
  assert.equal(source.name, "benchmark installed disk");
  assert.deepEqual(source.bytes, snapshot);
  assert.deepEqual(logs, [
    "Fetching benchmark installed disk",
    "Fetching /web/media/benchmark-installed.wbdisk",
    "Fetched benchmark installed disk (64.0 KiB)",
  ]);
});

test("benchmark disk loader rejects failed HTTP responses", async () => {
  await assert.rejects(
    () =>
      loadWithResponse({
        arrayBuffer: async () => new ArrayBuffer(0),
        headers: new Headers({ "content-length": "0" }),
        ok: false,
        status: 404,
      }),
    /Benchmark disk fetch failed: HTTP 404/,
  );
});

test("benchmark disk loader rejects missing and unexpected Content-Length before reading", async () => {
  let reads = 0;
  for (const value of [undefined, "2"]) {
    await assert.rejects(
      () =>
        loadWithResponse(
          {
            arrayBuffer: async () => {
              reads += 1;
              return new ArrayBuffer(0);
            },
            headers: new Headers(value === undefined ? {} : { "content-length": value }),
            ok: true,
            status: 200,
          },
          1,
        ),
      /valid Content-Length|does not match fixed fixture/,
    );
  }
  assert.equal(reads, 0);
});

test("benchmark disk loader rejects invalid magic", async () => {
  const snapshot = validSnapshot();
  snapshot[0] = 0;

  await assert.rejects(() => loadWithResponse(fakeResponse(snapshot)), /not a WBDISK01 snapshot/);
});

test("benchmark disk loader rejects a snapshot length inconsistent with its header", async () => {
  const snapshot = validSnapshot();
  new DataView(snapshot.buffer).setBigUint64(20, 2n, true);

  await assert.rejects(
    () => loadWithResponse(fakeResponse(snapshot)),
    /length does not match its header/,
  );
});

function loadWithResponse(
  response,
  expectedBytes = Number(response.headers.get("content-length")) || 1,
) {
  return fetchInstalledDiskBenchmark(
    { log: () => {}, setStatus: () => {} },
    {
      expectedBytes,
      fetchImpl: async () => response,
      locationLike: { href: "http://localhost:8080/index.html" },
    },
  );
}

function fakeResponse(bytes) {
  return {
    arrayBuffer: async () =>
      bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength),
    headers: new Headers({ "content-length": String(bytes.byteLength) }),
    ok: true,
    status: 200,
  };
}

function validSnapshot() {
  const bytes = new Uint8Array(28 + 8 + 64 * 1024);
  bytes.set(new TextEncoder().encode("WBDISK01"));
  const header = new DataView(bytes.buffer);
  header.setBigUint64(8, 64n * 1024n, true);
  header.setUint32(16, 64 * 1024, true);
  header.setBigUint64(20, 1n, true);
  return bytes;
}
