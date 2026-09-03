import assert from "node:assert/strict";
import test from "node:test";
import { installWebboxVmDevtools } from "./devtools.js?v=20260903-virgl-capset1-r2";

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

test("devtools debug memory helpers pass bigint addresses", async () => {
  await withFakeDocument(async () => {
    const calls = [];
    installWebboxVmDevtools(
      () => ({
        debug_read_pa_u64: async (pa) => calls.push(["pa", pa]),
        debug_read_va_u64: async (va, coreId) => calls.push(["va", va, coreId]),
        debug_translate_va: async (va, coreId) => calls.push(["translate", va, coreId]),
      }),
      () => undefined,
    );

    await window.__webboxvm.debugReadPa64("0x42039380");
    await window.__webboxvm.debugReadVa64("0xffff800082039380", 1);
    await window.__webboxvm.debugTranslateVa(0x1000, 2);

    assert.deepEqual(calls, [
      ["pa", 0x42039380n],
      ["va", 0xffff800082039380n, 1],
      ["translate", 0x1000n, 2],
    ]);
  });
});

function withFakeDocument(run) {
  const previousDocument = globalThis.document;
  const previousWindow = globalThis.window;
  const document = fakeDocument();
  globalThis.document = document;
  globalThis.window = {};
  try {
    const result = run(document);
    if (result?.finally) {
      return result.finally(restoreGlobals);
    }
    restoreGlobals();
    return result;
  } catch (error) {
    restoreGlobals();
    throw error;
  }

  function restoreGlobals() {
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
