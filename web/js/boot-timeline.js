const UART_TAIL_LIMIT = 32768;
const FAST_INITRD_PATTERN = /WEBBOXVM_FAST_INITRD_ACTIVE/;
const CPU1_ONLINE_PATTERN =
  /(?:CPU1:\s+Booted secondary processor|smp:\s+Brought up[^\r\n]*\b2 CPUs\b|WEBBOXVM_CPU1_ONLINE)/i;
const LOGIN_PROMPT_PATTERN =
  /(?:^|[\r\n])[A-Za-z0-9][A-Za-z0-9._-]* login:[ \t]*(?:\r?$)/im;

const MILESTONE_LABELS = {
  "fast-initrd": "minimal initrd active",
  "cpu1-online": "CPU1 online",
  "login-prompt": "login prompt",
};

export class UartBootTimeline {
  #enabled = false;
  #foundCpu1 = false;
  #foundFastInitrd = false;
  #foundLogin = false;
  #now;
  #onMilestone;
  #startedAt = 0;
  #tail = "";

  constructor({ now = () => performance.now(), onMilestone = () => {} } = {}) {
    this.#now = now;
    this.#onMilestone = onMilestone;
  }

  start({ installedSystem = false } = {}) {
    this.#enabled = installedSystem;
    this.#foundCpu1 = false;
    this.#foundFastInitrd = false;
    this.#foundLogin = false;
    this.#startedAt = installedSystem ? this.#now() : 0;
    this.#tail = "";
  }

  observe(output) {
    if (!this.#enabled || !output) {
      return;
    }
    this.#tail = `${this.#tail}${output}`.slice(-UART_TAIL_LIMIT);
    if (!this.#foundFastInitrd && FAST_INITRD_PATTERN.test(this.#tail)) {
      this.#foundFastInitrd = true;
      this.#emit("fast-initrd");
    }
    if (!this.#foundCpu1 && CPU1_ONLINE_PATTERN.test(this.#tail)) {
      this.#foundCpu1 = true;
      this.#emit("cpu1-online");
    }
    if (!this.#foundLogin && LOGIN_PROMPT_PATTERN.test(this.#tail)) {
      this.#foundLogin = true;
      this.#emit("login-prompt");
    }
  }

  #emit(name) {
    this.#onMilestone({
      elapsedMs: Math.max(0, this.#now() - this.#startedAt),
      name,
    });
  }
}

export function formatBootPhase(label, durationMs) {
  return `Fast boot ${label}: ${formatDuration(durationMs)} ms`;
}

export function formatBootMilestone({ elapsedMs, name }) {
  const label = MILESTONE_LABELS[name] ?? name;
  return `Fast boot milestone ${label}: ${formatDuration(elapsedMs)} ms after kernel start`;
}

function formatDuration(value) {
  return Math.max(0, Number(value) || 0).toFixed(1);
}
