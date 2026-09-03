import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { TerminalWriter } from "./terminal-writer.js?v=20260903-virgl-viewport-r1";

const previousDocument = globalThis.document;
const previousRequestAnimationFrame = globalThis.requestAnimationFrame;
const previousSetTimeout = globalThis.setTimeout;
const previousClearTimeout = globalThis.clearTimeout;

afterEach(() => {
  if (previousDocument === undefined) delete globalThis.document;
  else globalThis.document = previousDocument;
  if (previousRequestAnimationFrame === undefined) delete globalThis.requestAnimationFrame;
  else globalThis.requestAnimationFrame = previousRequestAnimationFrame;
  globalThis.setTimeout = previousSetTimeout;
  globalThis.clearTimeout = previousClearTimeout;
});

test("terminal writer keeps one parse in flight without waiting for paint", () => {
  const document = fakeDocument();
  globalThis.document = document;
  const writes = [];
  const paints = [];
  const term = {
    scrollToBottom() {},
    write(output, done) {
      writes.push({ done, output });
    },
  };
  const writer = new TerminalWriter(term, {
    afterPaint: (callback) => paints.push(callback),
  });

  writer.write("a");
  writer.write("b");
  writer.write("c");
  assert.deepEqual(writes.map(({ output }) => output), ["a"]);

  writes[0].done();
  assert.deepEqual(writes.map(({ output }) => output), ["a", "bc"]);

  writes[1].done();
  assert.equal(paints.length, 2);
  paints.shift()("paint");
  paints.shift()("paint");
  assert.equal(document.probe.textContent, "abc");
});

test("terminal writer scrolls parsed output before recording its painted tail", () => {
  const document = fakeDocument();
  globalThis.document = document;
  const order = [];
  const writer = new TerminalWriter(
    {
      scrollToBottom() {
        order.push("scroll");
      },
      write(_output, callback) {
        callback();
      },
    },
    {
      afterPaint(callback) {
        order.push("paint");
        callback("paint");
      },
      autoScroll: () => true,
    },
  );

  writer.write("x");

  assert.deepEqual(order, ["scroll", "paint"]);
  assert.equal(document.probe.textContent, "x");
  assert.equal(document.probe.dataset.renderedVia, "paint");
});

test("terminal writer reset clears timestamps and excludes an old in-flight chunk", () => {
  const document = fakeDocument();
  globalThis.document = document;
  let done;
  let paint;
  const writer = new TerminalWriter(
    {
      scrollToBottom() {},
      write(_output, callback) {
        done = callback;
      },
    },
    { afterPaint: (callback) => (paint = callback) },
  );

  writer.write("old");
  document.probe.dataset.renderedAt = "123";
  document.probe.dataset.renderedVia = "paint";
  writer.reset();
  done();
  paint("paint");

  assert.equal(document.probe.textContent, "");
  assert.equal(document.probe.dataset.renderedAt, undefined);
  assert.equal(document.probe.dataset.renderedVia, undefined);
});

test("default paint marker waits for two animation frames", () => {
  const document = fakeDocument();
  globalThis.document = document;
  const frames = [];
  globalThis.requestAnimationFrame = (callback) => frames.push(callback);
  globalThis.setTimeout = () => 1;
  globalThis.clearTimeout = () => {};
  const writer = new TerminalWriter({
    scrollToBottom() {},
    write(_output, callback) {
      callback();
    },
  });

  writer.write("x");
  assert.equal(document.probe.textContent, "");
  assert.equal(frames.length, 1);

  frames.shift()();
  assert.equal(document.probe.textContent, "");
  assert.equal(frames.length, 1);

  frames.shift()();
  assert.equal(document.probe.textContent, "x");
  assert.equal(document.probe.dataset.renderedVia, "paint");
});

function fakeDocument() {
  const document = {
    body: { append: (probe) => (document.probe = probe) },
    createElement: () => new FakeElement(),
    createTextNode: (data) => new FakeText(data),
    probe: undefined,
  };
  return document;
}

class FakeElement {
  dataset = {};
  style = {};

  append(text) {
    this.text = text;
  }

  get textContent() {
    return this.text.data;
  }

  setAttribute() {}
}

class FakeText {
  constructor(data) {
    this.data = data;
  }
  appendData(value) {
    this.data += value;
  }
  deleteData(start, count) {
    this.data = `${this.data.slice(0, start)}${this.data.slice(start + count)}`;
  }
}
