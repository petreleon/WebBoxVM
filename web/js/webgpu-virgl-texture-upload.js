export function copyTextureIfChanged(previous, pixels) {
  if (previous?.byteLength === pixels.byteLength && previous.every((value, index) => value === pixels[index])) {
    return undefined;
  }
  return new Uint8Array(pixels);
}
