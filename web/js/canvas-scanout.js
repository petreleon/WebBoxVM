export class CanvasScanoutRenderer {
  #canvas;
  #context;
  #needsFull = true;

  constructor(canvas) {
    this.#canvas = canvas;
  }

  render(state, requestedDirty) {
    this.#context ||= this.#canvas.getContext("2d", { alpha: false });
    if (!this.#context) throw new Error("Canvas2D is not available");
    if (this.#canvas.width !== state.width || this.#canvas.height !== state.height) {
      this.#canvas.width = state.width;
      this.#canvas.height = state.height;
      this.#needsFull = true;
    }
    const dirty = this.#needsFull ? state.fullRect() : requestedDirty;
    const image = this.#context.createImageData(dirty.width, dirty.height);
    let destination = 0;
    for (let row = 0; row < dirty.height; row += 1) {
      let source = ((dirty.y + row) * state.width + dirty.x) * 4;
      for (let column = 0; column < dirty.width; column += 1) {
        image.data[destination] = state.pixels[source + 2];
        image.data[destination + 1] = state.pixels[source + 1];
        image.data[destination + 2] = state.pixels[source];
        image.data[destination + 3] = state.pixels[source + 3];
        source += 4;
        destination += 4;
      }
    }
    this.#context.putImageData(image, dirty.x, dirty.y);
    this.#needsFull = false;
  }

  reset() {
    this.#needsFull = true;
    if (this.#context) this.#context.clearRect(0, 0, this.#canvas.width, this.#canvas.height);
  }
}
