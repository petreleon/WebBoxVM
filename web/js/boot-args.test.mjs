import assert from "node:assert/strict";
import test from "node:test";
import {
  extraBootargsFromLocation,
  installedDiskBenchmarkFromLocation,
  normalizeExtraBootargs,
  stagedSmpRequestedFromLocation,
} from "./boot-args.js?v=20260904-virgl-material-batch-r1";

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

test("staged SMP URL control defaults on and accepts exact on/off values", () => {
  assert.equal(stagedSmpRequestedFromLocation(undefined), true);
  assert.equal(
    stagedSmpRequestedFromLocation({ href: "http://localhost:8080/?staged-smp=on" }),
    true,
  );
  assert.equal(
    stagedSmpRequestedFromLocation({ href: "http://localhost:8080/?staged-smp=off" }),
    false,
  );
});

test("staged SMP URL control rejects ambiguous values", () => {
  for (const value of ["false", "0", "ON", ""]) {
    assert.throws(
      () => stagedSmpRequestedFromLocation({
        href: `http://localhost:8080/?staged-smp=${encodeURIComponent(value)}`,
      }),
      /must be 'on' or 'off'/,
    );
  }
});
