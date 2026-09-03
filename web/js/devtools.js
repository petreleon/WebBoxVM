import { DEFAULT_JIT_ENABLED } from "./vm-worker/state.js?v=20260903-webgpu-virtio-r4";

export function installWebboxVmDevtools(getEmulator, getRunner) {
  const bridge = installDomBridge(getEmulator);
  window.__webboxvm = {
    metrics() {
      const emulator = getEmulator();
      if (!emulator) {
        return undefined;
      }
      return {
        jit: emulator.jit_stats?.(),
        pages: emulator.allocated_pages(),
        pc: emulator.pc(),
        steps: emulator.total_steps(),
        uart: emulator.uart_output_len(),
      };
    },
    pause() {
      getRunner()?.pause();
    },
    resume(stepSlice) {
      getRunner()?.resume(stepSlice);
    },
    currentInstruction() {
      return getEmulator()?.current_instruction?.();
    },
    debugTranslateVa(va, coreId = 0) {
      return getEmulator()?.debug_translate_va?.(BigInt(va), coreId);
    },
    debugReadVa64(va, coreId = 0) {
      return getEmulator()?.debug_read_va_u64?.(BigInt(va), coreId);
    },
    debugReadPa64(pa) {
      return getEmulator()?.debug_read_pa_u64?.(BigInt(pa));
    },
    setJitEnabled(enabled) {
      getEmulator()?.set_jit_enabled?.(enabled);
    },
    send(input) {
      getEmulator()?.send_uart_input(input);
    },
    sendBytes(input) {
      getEmulator()?.send_uart_bytes(input);
    },
  };
  return bridge;
}

function installDomBridge(getEmulator) {
  const root = document.createElement("form");
  root.dataset.testid = "webboxvm-devtools";
  root.style.cssText = [
    "position:fixed",
    "left:0",
    "top:0",
    "width:32px",
    "height:48px",
    "opacity:0.01",
    "display:grid",
    "grid-template-columns:repeat(2, 16px)",
    "grid-auto-rows:16px",
    "overflow:hidden",
    "z-index:2147483647",
  ].join(";");

  const textInput = makeTextarea("webboxvm-devtools-text");
  const sendText = makeButton("webboxvm-devtools-send");
  const bytesInput = makeTextarea("webboxvm-devtools-bytes");
  const sendBytes = makeButton("webboxvm-devtools-send-bytes");
  const jitEnabled = makeCheckbox("webboxvm-devtools-jit-enabled");
  const applyJit = makeButton("webboxvm-devtools-apply-jit");

  sendText.addEventListener("click", () => {
    getEmulator()?.send_uart_input(terminalInput(textInput.value));
  });
  sendBytes.addEventListener("click", () => {
    const bytes = parseBytes(bytesInput.value);
    if (bytes.length > 0) {
      getEmulator()?.send_uart_bytes(bytes);
    }
  });
  applyJit.addEventListener("click", () => {
    getEmulator()?.set_jit_enabled?.(jitEnabled.checked);
  });

  root.addEventListener("submit", (event) => event.preventDefault());
  root.append(textInput, sendText, bytesInput, sendBytes, jitEnabled, applyJit);
  document.body.append(root);
  return {
    jitEnabled: () => jitEnabled.checked,
  };
}

function terminalInput(input) {
  return input.replace(/\r?\n/g, "\r");
}

function makeTextarea(testId) {
  const textarea = document.createElement("textarea");
  textarea.dataset.testid = testId;
  textarea.tabIndex = -1;
  styleControl(textarea);
  return textarea;
}

function makeButton(testId) {
  const button = document.createElement("button");
  button.type = "button";
  button.dataset.testid = testId;
  button.tabIndex = -1;
  button.textContent = testId;
  styleControl(button);
  return button;
}

function makeCheckbox(testId) {
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = DEFAULT_JIT_ENABLED;
  checkbox.dataset.testid = testId;
  checkbox.tabIndex = -1;
  styleControl(checkbox);
  return checkbox;
}

function styleControl(element) {
  element.style.cssText = [
    "width:16px",
    "height:16px",
    "min-width:0",
    "padding:0",
    "font-size:1px",
  ].join(";");
}

function parseBytes(input) {
  return new Uint8Array(input
    .split(/[\s,]+/)
    .filter(Boolean)
    .map((token) => Number.parseInt(token, token.startsWith("0x") ? 16 : 10))
    .filter((value) => Number.isInteger(value) && value >= 0 && value <= 255));
}
