import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { VmRunner } from "./runner.js?v=20260720-input-latency-r4";

const previousDocument = globalThis.document;

afterEach(() => {
  if (previousDocument === undefined) delete globalThis.document;
  else globalThis.document = previousDocument;
});

test("runner ignores callbacks from replaced and stopped emulators", async () => {
  globalThis.document = fakeDocument();
  const oldEmulator = fakeEmulator();
  const newEmulator = fakeEmulator();
  let current = oldEmulator;
  let errors = 0;
  let metrics = 0;
  let saves = 0;
  const milestones = [];
  const term = {
    output: "",
    scrollToBottom() {},
    write(value, done) {
      this.output += value;
      done?.();
    },
  };
  const runner = new VmRunner({
    disk: { shouldAutosave: () => true },
    els: {
      autoScroll: { checked: false },
      stepSlice: { addEventListener() {}, value: "1000" },
    },
    getEmulator: () => current,
    handleError: () => {
      errors += 1;
    },
    onBootTimeline: (milestone) => milestones.push(milestone),
    saveDisk: async () => {
      saves += 1;
    },
    term,
    ui: {
      log() {},
      updateMetrics() {
        metrics += 1;
      },
    },
  });
  runner.start({ installedSystem: true, stagedSmp: true });
  current = newEmulator;
  runner.start({ installedSystem: true, stagedSmp: true });

  oldEmulator.onUart("WEBBOXVM_CPU1_ONLINE\r\nold login: ");
  oldEmulator.onError(new Error("stale"));
  oldEmulator.onMetrics();
  oldEmulator.onNetwork("online");
  oldEmulator.onAutosave();
  await Promise.resolve();

  assert.deepEqual({ errors, metrics, saves }, { errors: 0, metrics: 0, saves: 0 });
  assert.equal(term.output, "");
  assert.deepEqual(milestones, []);
  assert.equal(oldEmulator.transitions, 0);

  newEmulator.onUart("WEBBOXVM_CPU1_ONLINE\r\nnew login: ");
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(newEmulator.transitions, 1);
  assert.equal(metrics, 1);
  assert.equal(milestones.length, 2);

  runner.stop();
  const output = term.output;
  newEmulator.onUart("late output");
  newEmulator.onError(new Error("late"));
  assert.equal(term.output, output);
  assert.equal(errors, 0);
});

function fakeEmulator() {
  return {
    start() {},
    stop() {},
    transitions: 0,
    transition_to_parallel() {
      this.transitions += 1;
      return { executionMode: "parallel-wasm", transitioned: true };
    },
  };
}

function fakeDocument() {
  return {
    body: { append() {} },
    createElement: () => ({ append() {}, dataset: {}, style: {} }),
    createTextNode: (value) => ({
      appendData(extra) {
        this.data += extra;
      },
      data: value,
      deleteData() {},
    }),
  };
}
