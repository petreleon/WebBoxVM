import assert from "node:assert/strict";
import test from "node:test";
import { DiskPersistence } from "./persistence.js?v=20260718-staged-fast-boot";

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

test("default background autosave avoids startup snapshot churn", () => {
  let now = 600_000 - 1;
  const disk = new DiskPersistence({ now: () => now, store: fakeStore() });
  const emulator = fakeEmulator();
  disk.available = true;
  emulator.generation = 1n;

  assert.equal(disk.shouldAutosave(emulator), false);

  now = 600_000;
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

test("autosave quota errors suspend later background saves", async () => {
  const quotaError = new Error("quota");
  quotaError.name = "QuotaExceededError";
  const logs = [];
  const disk = new DiskPersistence({
    autosaveIntervalMs: 1000,
    now: () => 1000,
    store: fakeStore({
      size: async () => 512,
      write: async () => {
        throw quotaError;
      },
    }),
  });
  const emulator = fakeEmulator();
  disk.available = true;
  emulator.generation = 1n;

  await disk.save(emulator, { quiet: true, log: (message) => logs.push(message) });

  assert.equal(disk.persistedBytes, 512);
  assert.match(logs[0], /Autosave paused/);
  emulator.generation = 2n;
  assert.equal(disk.shouldAutosave(emulator), false);
});

test("forced save quota errors still propagate", async () => {
  const quotaError = new Error("quota");
  quotaError.name = "QuotaExceededError";
  const disk = new DiskPersistence({
    store: fakeStore({
      write: async () => {
        throw quotaError;
      },
    }),
  });
  const emulator = fakeEmulator();
  disk.available = true;
  emulator.generation = 1n;

  await assert.rejects(() => disk.save(emulator, { force: true }), /quota/);
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
