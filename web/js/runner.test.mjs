import assert from "node:assert/strict";
import test, { afterEach, beforeEach } from "node:test";
import { VmRunner } from "./runner.js?v=20260904-virgl-depth-vertex-color-r1";

const previousDocument = globalThis.document;
let fakeDocument;

beforeEach(() => {
  fakeDocument = new FakeDocument();
  globalThis.document = fakeDocument;
});

afterEach(() => {
  if (previousDocument === undefined) {
    delete globalThis.document;
  } else {
    globalThis.document = previousDocument;
  }
});

test("uart probe appends small chunks without full text replacement", () => {
  const { emulator, runner, term } = setupRunner();
  runner.start();
  emulator.onUart("abc");
  emulator.onUart("def");

  assert.equal(term.output, "abcdef");
  assert.equal(fakeDocument.probe.textContent, "abcdef");
  assert.equal(fakeDocument.probe.textContentWrites, 0);
  assert.equal(fakeDocument.text.appendCalls, 2);
  assert.equal(fakeDocument.text.dataWrites, 0);
});

test("uart probe trims old text incrementally", () => {
  const { emulator, runner } = setupRunner();
  const first = "a".repeat(32760);
  const second = "bcdefghijk";

  runner.start();
  emulator.onUart(first);
  emulator.onUart(second);

  assert.equal(fakeDocument.probe.textContent, `${first.slice(2)}${second}`);
  assert.equal(fakeDocument.text.deleteCalls, 1);
  assert.equal(fakeDocument.text.appendCalls, 2);
  assert.equal(fakeDocument.text.dataWrites, 0);
});

test("uart probe replaces once for oversized chunks", () => {
  const { emulator, runner } = setupRunner();
  const output = "x".repeat(33000);

  runner.start();
  emulator.onUart(output);

  assert.equal(fakeDocument.probe.textContent, output.slice(-32768));
  assert.equal(fakeDocument.text.appendCalls, 0);
  assert.equal(fakeDocument.text.dataWrites, 1);
});

test("runner uses faster default step slice when input is blank", () => {
  const { emulator, runner } = setupRunner({ stepSlice: "" });

  runner.start();

  assert.equal(emulator.startedWith, 5_000_000);
});

test("runner forwards installed UART milestones and resets its probe on restart", () => {
  let now = 0;
  const milestones = [];
  const { emulator, runner } = setupRunner({
    now: () => now,
    onBootTimeline: (milestone) => milestones.push(milestone),
  });
  runner.start({ installedSystem: true });
  now = 20;
  emulator.onUart("CPU1: Booted secondary processor\r\n");
  runner.start({ installedSystem: true });
  assert.equal(fakeDocument.probe.textContent, "");
  now = 50;
  emulator.onUart("webboxvm login: ");

  assert.deepEqual(milestones, [
    { elapsedMs: 20, name: "cpu1-online" },
    { elapsedMs: 30, name: "login-prompt" },
  ]);
});

function setupRunner({ stepSlice = "1000000", now, onBootTimeline } = {}) {
  const emulator = { start: (value) => (emulator.startedWith = value) };
  const term = {
    output: "",
    scrollToBottom: () => {},
    write(output, done) {
      this.output += output;
      done?.();
    },
  };
  const runner = new VmRunner({
    disk: { shouldAutosave: () => false },
    els: {
      autoScroll: { checked: false },
      stepSlice: { addEventListener: () => {}, value: stepSlice },
    },
    getEmulator: () => emulator,
    handleError: () => {},
    now,
    onBootTimeline,
    saveDisk: async () => {},
    term,
    ui: { log: () => {}, updateMetrics: () => {} },
  });
  return { emulator, runner, term };
}

class FakeDocument {
  constructor() {
    this.body = { append: (probe) => (this.probe = probe) };
    this.probe = undefined;
    this.text = undefined;
  }

  createElement() {
    return new FakeElement();
  }

  createTextNode(data) {
    this.text = new FakeTextNode(data);
    return this.text;
  }
}

class FakeElement {
  dataset = {};
  style = {};
  textContentWrites = 0;

  append(child) {
    this.child = child;
  }

  get textContent() {
    return this.child?.data ?? "";
  }

  set textContent(value) {
    this.textContentWrites += 1;
    this.child.data = value;
  }
}

class FakeTextNode {
  appendCalls = 0;
  dataWrites = 0;
  deleteCalls = 0;

  constructor(data) {
    this.value = data;
  }

  get data() {
    return this.value;
  }

  set data(value) {
    this.dataWrites += 1;
    this.value = value;
  }

  appendData(value) {
    this.appendCalls += 1;
    this.value += value;
  }

  deleteData(start, count) {
    this.deleteCalls += 1;
    this.value = `${this.value.slice(0, start)}${this.value.slice(start + count)}`;
  }
}
