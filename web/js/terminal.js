import { delay } from "./utils.js?v=20260718-staged-fast-boot";

export async function waitForTerminal() {
  while (!window.Terminal || !window.FitAddon) {
    await delay(16);
  }
}

export function mountTerminal(els, getEmulator) {
  const term = new window.Terminal({
    cursorBlink: true,
    convertEol: true,
    fontFamily: "SFMono-Regular, Menlo, Consolas, monospace",
    fontSize: 13,
    letterSpacing: 0,
    scrollback: 8000,
    theme: {
      background: "#050606",
      foreground: "#edf1ee",
      cursor: "#5cc8a7",
      selectionBackground: "#365950",
      black: "#0b0d0d",
      red: "#df6b62",
      green: "#5cc8a7",
      yellow: "#e4b75a",
      blue: "#79a8d8",
      magenta: "#c98bd6",
      cyan: "#70c6d1",
      white: "#edf1ee",
      brightBlack: "#636d68",
      brightRed: "#f08379",
      brightGreen: "#77d9bb",
      brightYellow: "#f1c86b",
      brightBlue: "#95bee8",
      brightMagenta: "#d8a0e3",
      brightCyan: "#8bd9e2",
      brightWhite: "#ffffff",
    },
  });
  const fitAddon = new window.FitAddon.FitAddon();
  term.loadAddon(fitAddon);
  term.open(els.terminal);

  const fit = () => requestAnimationFrame(() => tryFit(fitAddon));
  fit();
  term.onData((data) => getEmulator()?.send_uart_input(data));
  window.addEventListener("resize", fit);
  return term;
}

function tryFit(fitAddon) {
  try {
    fitAddon.fit();
  } catch {
    // The terminal can be measured only after layout has settled.
  }
}
