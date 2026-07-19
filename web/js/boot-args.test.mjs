import assert from "node:assert/strict";
import test from "node:test";
import {
  extraBootargsFromLocation,
  installedDiskBenchmarkFromLocation,
  normalizeExtraBootargs,
} from "./boot-args.js?v=20260718-staged-fast-boot";

test("normalizeExtraBootargs trims and collapses whitespace", () => {
  assert.equal(normalizeExtraBootargs("  ftrace_filter=close   quiet "), "ftrace_filter=close quiet");
});

test("extraBootargsFromLocation decodes URL encoded bootargs", () => {
  const args = extraBootargsFromLocation({
    href: "http://localhost:8080/?bootargs=ftrace_filter%3Dclose+quiet",
  });

  assert.equal(args, "ftrace_filter=close quiet");
});

test("extraBootargsFromLocation is empty when missing", () => {
  assert.equal(extraBootargsFromLocation({ href: "http://localhost:8080/" }), "");
});

test("installed disk benchmark requires the exact opt-in value", () => {
  assert.equal(
    installedDiskBenchmarkFromLocation({
      href: "http://localhost:8080/?benchmark=installed-disk",
    }),
    true,
  );
  assert.equal(
    installedDiskBenchmarkFromLocation({
      href: "http://localhost:8080/?benchmark=https%3A%2F%2Fexample.com%2Fdisk.wbdisk",
    }),
    false,
  );
  assert.equal(
    installedDiskBenchmarkFromLocation({
      href: "http://localhost:8080/?disk_url=https%3A%2F%2Fexample.com%2Fdisk.wbdisk",
    }),
    false,
  );
  assert.equal(installedDiskBenchmarkFromLocation({ href: "http://localhost:8080/" }), false);
  assert.equal(installedDiskBenchmarkFromLocation(undefined), false);
});
