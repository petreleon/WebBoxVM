import assert from "node:assert/strict";
import test from "node:test";
import { bootPreparedInstalledDisk } from "./installed-boot.js?v=20260904-virgl-depth-r1";

test("installed boot boundary passes the successful preflight unchanged", () => {
  const calls = [];
  const constructed = [];
  class FakeEmulator {
    constructor(cores) {
      this.cores = cores;
      constructed.push(this);
    }
    boot_installed_disk_with_staged_smp(...args) {
      calls.push(args);
      return "booted";
    }
  }

  const boot = bootPreparedInstalledDisk(
    FakeEmulator,
    new Uint8Array([1, 2]),
    "debug",
    { bootCores: 2, parallelReady: true },
  );

  assert.equal(boot.result, "booted");
  assert.equal(boot.emulator, constructed[0]);
  assert.equal(boot.emulator.cores, 2);
  assert.deepEqual(calls, [[new Uint8Array([1, 2]), 2, "debug", true]]);
});

test("installed boot boundary preserves two cores but never stages after failed preflight", () => {
  const calls = [];
  class FakeEmulator {
    constructor(cores) {
      this.cores = cores;
    }
    boot_installed_disk_with_staged_smp(...args) {
      calls.push(args);
      return "booted";
    }
  }

  const { emulator } = bootPreparedInstalledDisk(
    FakeEmulator,
    new Uint8Array([3]),
    "",
    { bootCores: 2, parallelReady: false },
  );

  assert.equal(emulator.cores, 2);
  assert.deepEqual(calls, [[new Uint8Array([3]), 2, "", false]]);
});

test("explicit opt-out reaches Rust despite a successful worker preflight", () => {
  const calls = [];
  class FakeEmulator {
    boot_installed_disk_with_staged_smp(...args) {
      calls.push(args);
      return "booted";
    }
  }

  bootPreparedInstalledDisk(
    FakeEmulator,
    new Uint8Array([4]),
    "",
    { bootCores: 2, parallelReady: true },
    false,
  );

  assert.deepEqual(calls, [[new Uint8Array([4]), 2, "", false]]);
});
