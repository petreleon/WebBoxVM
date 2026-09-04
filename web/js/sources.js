import { formatBytes, nextFrame } from "./utils.js?v=20260904-virgl-material-batch-r1";

const BENCHMARK_DISK_PATH = "./media/benchmark-installed.wbdisk";
const BENCHMARK_DISK_BYTES = 1_259_034_724;
const WBDISK_CHUNK_BYTES = 64 * 1024;
const WBDISK_ENTRY_HEADER_BYTES = 8;
const WBDISK_HEADER_BYTES = 28;
const WBDISK_MAGIC = new Uint8Array([0x57, 0x42, 0x44, 0x49, 0x53, 0x4b, 0x30, 0x31]);

export async function readSelectedIso(els, ui) {
  const file = els.isoFile.files?.[0];
  if (!file) {
    ui.setStatus("No ISO selected", "warn");
    ui.log("No ISO selected");
    return undefined;
  }

  ui.setStatus(`Reading ${file.name}`);
  ui.log(`Reading ${file.name} (${formatBytes(file.size)})`);
  await nextFrame();

  const buffer = await file.arrayBuffer();
  return {
    bytes: new Uint8Array(buffer),
    name: file.name,
  };
}

export async function fetchBundledDebian(ui) {
  const url = "./media/debian-arm64-netinst.iso";
  ui.setStatus("Fetching Debian ISO");
  ui.log(`Fetching ${url}`);
  await nextFrame();

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Debian ISO fetch failed: HTTP ${response.status}`);
  }
  const buffer = await response.arrayBuffer();
  return {
    bytes: new Uint8Array(buffer),
    name: "Debian arm64 netinst",
  };
}

export async function fetchInstalledDiskBenchmark(
  ui,
  {
    expectedBytes = BENCHMARK_DISK_BYTES,
    fetchImpl = globalThis.fetch,
    locationLike = globalThis.location,
  } = {},
) {
  const pageUrl = new URL(locationLike.href);
  const url = new URL(BENCHMARK_DISK_PATH, pageUrl);
  if (url.origin !== pageUrl.origin) {
    throw new Error("Benchmark disk must use the page origin");
  }

  ui.setStatus("Fetching benchmark installed disk");
  ui.log(`Fetching ${url.pathname}`);
  await nextFrame();

  const response = await fetchImpl(url.href, {
    cache: "no-store",
    credentials: "same-origin",
    mode: "same-origin",
  });
  if (!response.ok) {
    throw new Error(`Benchmark disk fetch failed: HTTP ${response.status}`);
  }

  const contentLength = parseContentLength(
    response.headers.get("content-length"),
    expectedBytes,
  );
  const buffer = await response.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  if (bytes.byteLength !== contentLength) {
    throw new Error(
      `Benchmark disk length differs from Content-Length (${bytes.byteLength} != ${contentLength})`,
    );
  }
  validateWbdiskSnapshot(bytes);
  ui.log(`Fetched benchmark installed disk (${formatBytes(bytes.byteLength)})`);
  return {
    bytes,
    name: "benchmark installed disk",
  };
}

function parseContentLength(value, expectedBytes) {
  if (!/^\d+$/.test(value ?? "")) {
    throw new Error("Benchmark disk response needs a valid Content-Length");
  }
  const length = Number(value);
  if (!Number.isSafeInteger(length) || length !== expectedBytes) {
    throw new Error(
      `Benchmark disk Content-Length does not match fixed fixture (${length} != ${expectedBytes})`,
    );
  }
  return length;
}

function validateWbdiskSnapshot(bytes) {
  if (bytes.byteLength < WBDISK_HEADER_BYTES || !startsWith(bytes, WBDISK_MAGIC)) {
    throw new Error("Benchmark disk is not a WBDISK01 snapshot");
  }
  const header = new DataView(bytes.buffer, bytes.byteOffset, WBDISK_HEADER_BYTES);
  const chunkBytes = header.getUint32(16, true);
  if (chunkBytes !== WBDISK_CHUNK_BYTES) {
    throw new Error(`Benchmark disk has unsupported chunk size ${chunkBytes}`);
  }
  const chunkCount = header.getBigUint64(20, true);
  const expectedLength =
    BigInt(WBDISK_HEADER_BYTES) +
    chunkCount * BigInt(WBDISK_ENTRY_HEADER_BYTES + chunkBytes);
  if (expectedLength !== BigInt(bytes.byteLength)) {
    throw new Error(
      `Benchmark disk length does not match its header (${bytes.byteLength} != ${expectedLength})`,
    );
  }
}

function startsWith(bytes, prefix) {
  return prefix.every((byte, index) => bytes[index] === byte);
}
