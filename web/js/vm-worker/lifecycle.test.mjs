import assert from "node:assert/strict";
import test, { afterEach } from "node:test";
import { metrics } from "./lifecycle.js";
import { state } from "./state.js";

afterEach(() => {
  state.emulator = undefined;
});

test("routine metrics avoid current instruction decoding", () => {
  let decoded = false;
  state.emulator = {
    allocated_pages: () => 1,
    current_instruction: () => {
      decoded = true;
      return "{}";
    },
    install_disk_allocated_bytes: () => 2n,
    install_disk_generation: () => 3n,
    install_disk_size_bytes: () => 4n,
    network_rx_packets: () => 5n,
    network_tx_packets: () => 6n,
    network_tx_pending: () => 7,
    pc: () => 8n,
    total_steps: () => 9n,
    uart_output_len: () => 10,
  };

  const snapshot = metrics();

  assert.equal(decoded, false);
  assert.equal(snapshot.currentInstruction, undefined);
  assert.equal(snapshot.totalSteps, 9n);
});
