import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { VmRunner } from "./runner.js?v=20260904-virgl-depth-batch-compare-r1";

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
  let resets = 0;
  let saves = 0;
  const frames = [];
  const frames3d = [];
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
    display: {
      present: (packet) => frames.push(packet),
      present3d: (packet) => {
        frames3d.push(packet);
        return { sequence: 4, success: true };
      },
      reset: () => { resets += 1; },
    },
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
  oldEmulator.onGpuFrame("old frame");
  oldEmulator.onGpu3dFrame("old 3D frame");
  oldEmulator.onGpuReset();
  oldEmulator.onNetwork("online");
  oldEmulator.onAutosave();
  await Promise.resolve();

  assert.deepEqual({ errors, metrics, saves }, { errors: 0, metrics: 0, saves: 0 });
  assert.equal(term.output, "");
  assert.deepEqual(milestones, []);
  assert.deepEqual(frames, []);
  assert.deepEqual(frames3d, []);
  assert.equal(oldEmulator.transitions, 0);
  assert.equal(resets, 0);

  newEmulator.onUart("WEBBOXVM_CPU1_ONLINE\r\nnew login: ");
  newEmulator.onGpuFrame("new frame");
  newEmulator.onGpu3dFrame("new 3D frame");
  newEmulator.onGpuReset();
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(newEmulator.transitions, 1);
  assert.equal(metrics, 1);
  assert.equal(milestones.length, 2);
  assert.deepEqual(frames, ["new frame"]);
  assert.deepEqual(frames3d, ["new 3D frame"]);
  assert.deepEqual(newEmulator.acks, [[4, true]]);
  assert.equal(resets, 1);

  runner.stop();
  const output = term.output;
  newEmulator.onUart("late output");
  newEmulator.onGpuFrame("late frame");
  newEmulator.onGpu3dFrame("late 3D frame");
  newEmulator.onGpuReset();
  newEmulator.onError(new Error("late"));
  assert.equal(term.output, output);
  assert.equal(errors, 0);
  assert.deepEqual(frames, ["new frame"]);
  assert.deepEqual(frames3d, ["new 3D frame"]);
  assert.deepEqual(newEmulator.acks, [[4, true]]);
  assert.equal(resets, 1);
});

function fakeEmulator() {
  return {
    acks: [],
    gpu3d_ack(sequence, success) { this.acks.push([sequence, success]); },
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
