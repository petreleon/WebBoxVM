import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import {
  checkWebAssetGraph,
  findRelativeModuleSpecifiers,
} from "../../scripts/stamp_web_asset_version.mjs?v=20260904-virgl-depth-batch-compare-r1";
import { WEBBOXVM_ASSET_VERSION, versionedUrl } from "./asset-version.js?v=20260904-virgl-depth-batch-compare-r1";

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

test("module graph finder covers every ESM dependency form", () => {
  const keyword = ["im", "port"].join("");
  const source = [
    `${keyword} value from "./static.js";`,
    `${keyword}("./dynamic.js");`,
    `${keyword} "./side-effect.js";`,
  ].join("\n");

  assert.deepEqual(findRelativeModuleSpecifiers(source), [
    "./static.js",
    "./dynamic.js",
    "./side-effect.js",
  ]);
});

test("all local web and test module edges share one asset version", async () => {
  assert.deepEqual(await checkWebAssetGraph(), []);
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
