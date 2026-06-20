import assert from "node:assert/strict";
import test from "node:test";
import { DiskPersistence } from "./persistence.js";

test("background autosave throttles unchanged and recent generations", () => {
  let now = 0;
  const disk = new DiskPersistence({
    autosaveIntervalMs: 1000,
    now: () => now,
    store: fakeStore(),
  });
  const emulator = fakeEmulator();
  disk.available = true;

  assert.equal(disk.shouldAutosave(emulator), false);

  emulator.generation = 1n;
  now = 999;
  assert.equal(disk.shouldAutosave(emulator), false);

  now = 1000;
  assert.equal(disk.shouldAutosave(emulator), true);
  assert.equal(disk.shouldAutosave(emulator), false);

  emulator.generation = 2n;
  now = 1500;
  assert.equal(disk.shouldAutosave(emulator), false);

  now = 2000;
  assert.equal(disk.shouldAutosave(emulator), true);
});

test("background autosave skips while a snapshot is already saving", () => {
  const disk = new DiskPersistence({
    autosaveIntervalMs: 1000,
    now: () => 5000,
    store: fakeStore(),
  });
  const emulator = fakeEmulator();
  disk.available = true;
  disk.saving = true;
  emulator.generation = 1n;

  assert.equal(disk.shouldAutosave(emulator), false);
});

test("forced saves still write even inside the autosave interval", async () => {
  let writes = 0;
  const store = fakeStore({
    write: async () => {
      writes += 1;
    },
  });
  const disk = new DiskPersistence({
    autosaveIntervalMs: 60_000,
    now: () => 0,
    store,
  });
  const emulator = fakeEmulator();
  disk.available = true;
  emulator.generation = 1n;

  await disk.save(emulator, { force: true, quiet: true });

  assert.equal(writes, 1);
});

function fakeEmulator() {
  return {
    generation: 0n,
    install_disk_generation() {
      return this.generation;
    },
    install_disk_snapshot() {
      return new Uint8Array([Number(this.generation)]);
    },
  };
}

function fakeStore(overrides = {}) {
  return {
    clear: async () => {},
    load: async () => undefined,
    requestPersistence: async () => {},
    size: async () => 0,
    write: async () => {},
    ...overrides,
  };
}
