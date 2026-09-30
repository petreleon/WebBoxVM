/** Execute a queued GPU draw or control packet against its owning display epoch. */
export async function executeGpu3dCommand(renderer, frame, isCurrent, acquireBackend) {
  if (frame.protocol === "virgl-resident-release") {
    if (isCurrent()) renderer.release(frame);
    return {};
  }
  const backend = await acquireBackend();
  if (!isCurrent()) return { sequence: frame.sequence, success: false };
  if (!backend) throw new Error("Experimental guest 3D requires WebGPU; Canvas2D is 2D-only");
  const rendered = await renderer.render(backend, frame, isCurrent);
  if (!rendered || !isCurrent()) return { sequence: frame.sequence, success: false };
  return {
    sequence: frame.sequence, success: true,
    ...(rendered.readback && { readback: rendered.readback }),
    ...(rendered.resident && { resident: true }),
  };
}
