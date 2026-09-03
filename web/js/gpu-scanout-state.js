export class GpuScanoutState {
  dirty;
  height = 0;
  pixels;
  width = 0;

  apply(frame) {
    if (!this.pixels || frame.scanoutWidth !== this.width || frame.scanoutHeight !== this.height) {
      this.width = frame.scanoutWidth;
      this.height = frame.scanoutHeight;
      this.pixels = new Uint8Array(this.width * this.height * 4);
      this.dirty = this.fullRect();
    }
    const rowBytes = frame.width * 4;
    for (let row = 0; row < frame.height; row += 1) {
      const source = row * rowBytes;
      const destination = ((frame.y + row) * this.width + frame.x) * 4;
      this.pixels.set(frame.pixels.subarray(source, source + rowBytes), destination);
    }
    this.dirty = unionRect(this.dirty, frame);
  }

  takeDirty(forceFull = false) {
    const dirty = forceFull && this.pixels ? this.fullRect() : this.dirty;
    this.dirty = undefined;
    return dirty;
  }

  markFull() {
    if (this.pixels) this.dirty = this.fullRect();
  }

  extract(rect) {
    const rowBytes = rect.width * 4;
    const result = new Uint8Array(rowBytes * rect.height);
    for (let row = 0; row < rect.height; row += 1) {
      const source = ((rect.y + row) * this.width + rect.x) * 4;
      result.set(this.pixels.subarray(source, source + rowBytes), row * rowBytes);
    }
    return result;
  }

  fullRect() {
    return { height: this.height, width: this.width, x: 0, y: 0 };
  }

  reset() {
    this.dirty = undefined;
    this.height = 0;
    this.pixels = undefined;
    this.width = 0;
  }
}

function unionRect(first, second) {
  if (!first) return { height: second.height, width: second.width, x: second.x, y: second.y };
  const x = Math.min(first.x, second.x);
  const y = Math.min(first.y, second.y);
  const right = Math.max(first.x + first.width, second.x + second.width);
  const bottom = Math.max(first.y + first.height, second.y + second.height);
  return { height: bottom - y, width: right - x, x, y };
}
