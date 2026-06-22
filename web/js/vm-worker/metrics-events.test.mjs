import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { maybeRequestAutosave } from "./metrics-events.js";
import { AUTOSAVE_INTERVAL_MS, AUTOSAVE_POLL_MS, state } from "./state.js";

const previousPostMessage = globalThis.postMessage;
let messages = [];

beforeEach(() => {
  messages = [];
  globalThis.postMessage = (message) => messages.push(message);
});

afterEach(() => {
  globalThis.postMessage = previousPostMessage;
  state.emulator = undefined;
  state.lastAutosaveAt = 0;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = 0;
});

test("autosave skips disk generation polling inside poll window", () => {
  let generationPolls = 0;
  const now = performance.now();
  state.lastAutosaveAt = now - AUTOSAVE_INTERVAL_MS - 10;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = now;
  state.emulator = {
    install_disk_generation: () => {
      generationPolls += 1;
      return 1n;
    },
  };

  maybeRequestAutosave();

  assert.equal(generationPolls, 0);
  assert.deepEqual(messages, []);
});

test("autosave polls generation and requests save after intervals", () => {
  let generationPolls = 0;
  const now = performance.now();
  state.lastAutosaveAt = now - AUTOSAVE_INTERVAL_MS - 10;
  state.lastAutosaveGeneration = 0n;
  state.lastAutosavePollAt = now - AUTOSAVE_POLL_MS - 10;
  state.emulator = {
    install_disk_generation: () => {
      generationPolls += 1;
      return 1n;
    },
  };

  maybeRequestAutosave();

  assert.equal(generationPolls, 1);
  assert.equal(state.lastAutosaveGeneration, 1n);
  assert.deepEqual(messages, [{ event: "autosave" }]);
});
