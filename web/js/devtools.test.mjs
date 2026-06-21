import assert from "node:assert/strict";
import test from "node:test";
import { installWebboxVmDevtools } from "./devtools.js";

test("hidden devtools jit checkbox follows the safe default", () => {
  withFakeDocument((document) => {
    const devtools = installWebboxVmDevtools(() => undefined, () => undefined);
    const checkbox = document.findByTestId("webboxvm-devtools-jit-enabled");

    assert.equal(checkbox.checked, false);
    assert.equal(devtools.jitEnabled(), false);
  });
});

test("hidden devtools jit checkbox still allows manual enable", () => {
  withFakeDocument((document) => {
    let enabled;
    installWebboxVmDevtools(
      () => ({ set_jit_enabled: (value) => { enabled = value; } }),
      () => undefined,
    );

    document.findByTestId("webboxvm-devtools-jit-enabled").checked = true;
    document.findByTestId("webboxvm-devtools-apply-jit").dispatch("click");

    assert.equal(enabled, true);
  });
});

function withFakeDocument(run) {
  const previousDocument = globalThis.document;
  const previousWindow = globalThis.window;
  const document = fakeDocument();
  globalThis.document = document;
  globalThis.window = {};
  try {
    run(document);
  } finally {
    globalThis.document = previousDocument;
    globalThis.window = previousWindow;
  }
}

function fakeDocument() {
  const elements = [];
  const body = fakeElement("body", elements);
  return {
    body,
    createElement(tagName) {
      const element = fakeElement(tagName, elements);
      elements.push(element);
      return element;
    },
    findByTestId(testId) {
      return elements.find((element) => element.dataset.testid === testId);
    },
  };
}

function fakeElement(tagName, elements) {
  const listeners = new Map();
  return {
    children: [],
    dataset: {},
    style: {},
    tagName,
    addEventListener(type, listener) {
      listeners.set(type, listener);
    },
    append(...children) {
      this.children.push(...children);
      elements.push(...children);
    },
    dispatch(type) {
      listeners.get(type)?.({ preventDefault() {} });
    },
  };
}
