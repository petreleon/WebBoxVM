const MAX_CACHE_BYTES = 2 * 1024 * 1024;

export class VirglVertexUploadCache {
  #buffer;
  #bytes;

  upload(device, buffer, bytes) {
    if (this.#buffer === buffer && sameBytes(this.#bytes, bytes)) return false;
    device.queue.writeBuffer(buffer, 0, bytes);
    this.#buffer = buffer;
    this.#bytes = bytes.byteLength <= MAX_CACHE_BYTES ? new Uint8Array(bytes) : undefined;
    return true;
  }

  invalidate() {
    this.#buffer = undefined;
    this.#bytes = undefined;
  }
}

function sameBytes(previous, next) {
  if (!previous || previous.byteLength !== next.byteLength) return false;
  for (let index = 0; index < previous.byteLength; index += 1) {
    if (previous[index] !== next[index]) return false;
  }
  return true;
}
