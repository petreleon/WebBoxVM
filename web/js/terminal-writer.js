import { installTextProbe } from "./uart-probe.js?v=20260904-virgl-readback-pool-r1";

const RENDER_TAIL_LIMIT = 32768;

export class TerminalWriter {
  #afterPaint;
  #autoScroll;
  #generation = 0;
  #pending = [];
  #renderProbe;
  #renderText;
  #renderedTail = "";
  #term;
  #writing = false;

  constructor(term, { afterPaint = afterNextPaint, autoScroll = () => false } = {}) {
    this.#term = term;
    this.#afterPaint = afterPaint;
    this.#autoScroll = autoScroll;
    const installed = installTextProbe("webboxvm-render-tail");
    this.#renderProbe = installed.probe;
    this.#renderText = installed.text;
  }

  write(output) {
    if (!output) return;
    this.#pending.push(output);
    this.#flush();
  }

  reset() {
    this.#generation += 1;
    this.#pending = [];
    this.#renderedTail = "";
    this.#renderText.data = "";
    delete this.#renderProbe.dataset.renderedAt;
    delete this.#renderProbe.dataset.renderedVia;
  }

  #flush() {
    if (this.#writing || this.#pending.length === 0) return;
    const chunks = this.#pending;
    this.#pending = [];
    const output = chunks.length === 1 ? chunks[0] : chunks.join("");
    const generation = this.#generation;
    this.#writing = true;
    this.#term.write(output, () => {
      if (generation === this.#generation && this.#autoScroll()) {
        this.#term.scrollToBottom();
      }
      this.#afterPaint((renderedVia = "paint") => {
        if (generation === this.#generation) {
          this.#recordRendered(output, renderedVia);
        }
      });
      this.#writing = false;
      this.#flush();
    });
  }

  #recordRendered(output, renderedVia) {
    if (output.length >= RENDER_TAIL_LIMIT) {
      this.#renderedTail = output.slice(-RENDER_TAIL_LIMIT);
      this.#renderText.data = this.#renderedTail;
    } else {
      const overflow = Math.max(
        0,
        this.#renderedTail.length + output.length - RENDER_TAIL_LIMIT,
      );
      if (overflow > 0) {
        this.#renderedTail = this.#renderedTail.slice(overflow);
        this.#renderText.deleteData(0, overflow);
      }
      this.#renderedTail += output;
      this.#renderText.appendData(output);
    }
    this.#renderProbe.dataset.renderedAt = String(performance.now());
    this.#renderProbe.dataset.renderedVia = renderedVia;
  }
}

function afterNextPaint(callback) {
  if (typeof requestAnimationFrame !== "function") {
    callback("synchronous");
    return;
  }
  let finished = false;
  const finish = (renderedVia) => {
    if (finished) return;
    finished = true;
    clearTimeout(timeout);
    callback(renderedVia);
  };
  const timeout = setTimeout(() => finish("timeout"), 250);
  requestAnimationFrame(() => requestAnimationFrame(() => finish("paint")));
}
