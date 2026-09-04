import assert from "node:assert/strict";
import test from "node:test";
import {
  UartBootTimeline,
  formatBootMilestone,
  formatBootPhase,
} from "./boot-timeline.js?v=20260904-virgl-depth-batch-r1";

test("installed boot timeline finds split CPU1 and login milestones once", () => {
  let now = 1000;
  const milestones = [];
  const timeline = new UartBootTimeline({
    now: () => now,
    onMilestone: (milestone) => milestones.push(milestone),
  });
  timeline.start({ installedSystem: true });

  now = 1125.25;
  timeline.observe("[    0.10] CPU1: Booted secondary");
  timeline.observe(" processor 0x0000000001 [0x410fd034]\r\n");
  timeline.observe("[    0.11] CPU1: Booted secondary processor again\r\n");
  now = 1480.75;
  timeline.observe("Debian GNU/Linux 13 webboxvm ttyAMA0\r\n\r\nwebbox");
  timeline.observe("vm login: ");
  timeline.observe("\r\nwebboxvm login: ");

  assert.deepEqual(milestones, [
    { elapsedMs: 125.25, name: "cpu1-online" },
    { elapsedMs: 480.75, name: "login-prompt" },
  ]);
});

test("timeline ignores installer-like text unless an installed boot is active", () => {
  const milestones = [];
  const timeline = new UartBootTimeline({
    now: () => 10,
    onMilestone: (milestone) => milestones.push(milestone),
  });
  timeline.start();
  timeline.observe("CPU1: Booted secondary processor\r\ndebian login: ");
  timeline.start({ installedSystem: true });
  timeline.observe("Starting Login Service\r\nPrompt: login:");

  assert.deepEqual(milestones, []);
});

test("timeline accepts the verified late CPU marker", () => {
  const milestones = [];
  const timeline = new UartBootTimeline({
    now: () => 25,
    onMilestone: (milestone) => milestones.push(milestone),
  });
  timeline.start({ installedSystem: true });

  timeline.observe("WEBBOXVM_CPU1_ONLINE\r\n");

  assert.deepEqual(milestones, [{ elapsedMs: 0, name: "cpu1-online" }]);
});

test("timeline reports the minimal initrd marker once", () => {
  const milestones = [];
  const timeline = new UartBootTimeline({
    now: () => 25,
    onMilestone: (milestone) => milestones.push(milestone),
  });
  timeline.start({ installedSystem: true });

  timeline.observe("WEBBOXVM_FAST_INITRD_");
  timeline.observe("ACTIVE\r\nWEBBOXVM_FAST_INITRD_ACTIVE\r\n");

  assert.deepEqual(milestones, [{ elapsedMs: 0, name: "fast-initrd" }]);
});

test("boot timeline messages use stable one-decimal durations", () => {
  assert.equal(formatBootPhase("OPFS load", 12.34), "Fast boot OPFS load: 12.3 ms");
  assert.equal(
    formatBootMilestone({ elapsedMs: 45.67, name: "cpu1-online" }),
    "Fast boot milestone CPU1 online: 45.7 ms after kernel start",
  );
});
