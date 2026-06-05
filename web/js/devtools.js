export function installWebboxVmDevtools(getEmulator, getRunner) {
  installDomBridge(getEmulator);
  window.__webboxvm = {
    metrics() {
      const emulator = getEmulator();
      if (!emulator) {
        return undefined;
      }
      return {
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
}

function installDomBridge(getEmulator) {
  const root = document.createElement("form");
  root.dataset.testid = "webboxvm-devtools";
  root.style.cssText = [
    "position:fixed",
    "left:0",
    "top:0",
    "width:24px",
    "height:24px",
    "opacity:0.01",
    "overflow:visible",
    "z-index:2147483647",
  ].join(";");

  const textInput = makeTextarea("webboxvm-devtools-text");
  const sendText = makeButton("webboxvm-devtools-send");
  const bytesInput = makeTextarea("webboxvm-devtools-bytes");
  const sendBytes = makeButton("webboxvm-devtools-send-bytes");
  const jitEnabled = makeCheckbox("webboxvm-devtools-jit-enabled");
  const applyJit = makeButton("webboxvm-devtools-apply-jit");

  sendText.addEventListener("click", () => {
    getEmulator()?.send_uart_input(textInput.value);
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
}

function makeTextarea(testId) {
  const textarea = document.createElement("textarea");
  textarea.dataset.testid = testId;
  textarea.tabIndex = -1;
  return textarea;
}

function makeButton(testId) {
  const button = document.createElement("button");
  button.type = "button";
  button.dataset.testid = testId;
  button.tabIndex = -1;
  button.textContent = testId;
  return button;
}

function makeCheckbox(testId) {
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = true;
  checkbox.dataset.testid = testId;
  checkbox.tabIndex = -1;
  return checkbox;
}

function parseBytes(input) {
  return new Uint8Array(input
    .split(/[\s,]+/)
    .filter(Boolean)
    .map((token) => Number.parseInt(token, token.startsWith("0x") ? 16 : 10))
    .filter((value) => Number.isInteger(value) && value >= 0 && value <= 255));
}
