import { transferableBytes } from "./bytes.js?v=20260904-virgl-readback-pool-r1";

export function postGpu3dAck(channel, sequence, success, readback, resident = false) {
  const payload = { sequence, success: Boolean(success) };
  if (resident) {
    channel.post("gpu3dAck", { ...payload, resident: true });
    return;
  }
  if (!(readback?.pixels instanceof Uint8Array)) {
    channel.post("gpu3dAck", payload);
    return;
  }
  const pixels = transferableBytes(readback.pixels);
  channel.post("gpu3dAck", {
    ...payload, readback: { format: readback.format, pixels },
  }, [pixels.buffer]);
}
