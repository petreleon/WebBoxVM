export function virglColorTarget(format, blend, writeMask = 0xF, sourceOver) {
  return {
    ...(blend === "replace" ? {} : { blend: sourceOver }),
    ...(writeMask === 0xF ? {} : { writeMask }),
    format,
  };
}
