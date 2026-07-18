import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { WEBBOXVM_ASSET_VERSION, versionedUrl } from "./asset-version.js";

test("versionedUrl stamps the shared asset version", () => {
  const url = versionedUrl("./vm-worker.js", "http://localhost/app.js");

  assert.equal(url.href, `http://localhost/vm-worker.js?v=${WEBBOXVM_ASSET_VERSION}`);
});

test("versionedUrl preserves unrelated query parameters", () => {
  const url = versionedUrl("./emulator_bg.wasm?debug=1", "http://localhost/js/vm-worker/wasm.js");

  assert.equal(url.searchParams.get("debug"), "1");
  assert.equal(url.searchParams.get("v"), WEBBOXVM_ASSET_VERSION);
});

test("browser entrypoint uses the shared asset version", async () => {
  const html = await readFile(new URL("../index.html", import.meta.url), "utf8");

  assert.match(html, new RegExp(`app\\.js\\?v=${WEBBOXVM_ASSET_VERSION}`));
});

test("worker and wasm package URLs are cache busted", async () => {
  const workerVm = await readFile(new URL("./worker-vm.js", import.meta.url), "utf8");
  const wasmLoader = await readFile(new URL("./vm-worker/wasm.js", import.meta.url), "utf8");

  assert.match(workerVm, /versionedUrl\("\.\/vm-worker\.js"/);
  assert.match(wasmLoader, /loadPackage\("\.\.\/\.\.\/pkg-threaded"\)/);
  assert.match(wasmLoader, /loadPackage\("\.\.\/\.\.\/pkg"\)/);
  assert.match(wasmLoader, /versionedUrl\(`\$\{directory\}\/emulator\.js`/);
  assert.match(wasmLoader, /versionedUrl\(`\$\{directory\}\/emulator_bg\.wasm`/);
});
