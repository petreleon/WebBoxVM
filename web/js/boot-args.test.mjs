import assert from "node:assert/strict";
import test from "node:test";
import { extraBootargsFromLocation, normalizeExtraBootargs } from "./boot-args.js";

test("normalizeExtraBootargs trims and collapses whitespace", () => {
  assert.equal(normalizeExtraBootargs("  ftrace_filter=close   quiet "), "ftrace_filter=close quiet");
});

test("extraBootargsFromLocation decodes URL encoded bootargs", () => {
  const args = extraBootargsFromLocation({
    href: "http://localhost:8080/?bootargs=ftrace_filter%3Dclose+quiet",
  });

  assert.equal(args, "ftrace_filter=close quiet");
});

test("extraBootargsFromLocation is empty when missing", () => {
  assert.equal(extraBootargsFromLocation({ href: "http://localhost:8080/" }), "");
});
