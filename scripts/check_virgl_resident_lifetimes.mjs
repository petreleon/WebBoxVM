// With the web server open in Playwright CLI:
// playwright-cli run-code --filename scripts/check_virgl_resident_lifetimes.mjs
async (page) => {
  return page.evaluate(async () => {
    const { GuestDisplay } = await import("/js/gpu-display.js");
    const { virglClearPacket } = await import("/js/gpu-test-packets.mjs");
    const { virglResidentReadbackPacket } = await import("/js/gpu-test-virgl-resident-readback.mjs");
    const { virglResidentReleasePacket } = await import("/js/gpu-test-virgl-resident-release.mjs");
    const require = (condition, message) => { if (!condition) throw new Error(message); };
    require(navigator.gpu, "WebGPU is unavailable");
    const devices = []; const alive = new Set(); const errors = []; let created = 0; let destroyed = 0;
    const gpu = {
      getPreferredCanvasFormat: () => navigator.gpu.getPreferredCanvasFormat(),
      async requestAdapter(options) {
        const adapter = await navigator.gpu.requestAdapter(options);
        require(adapter, "WebGPU adapter is unavailable");
        return {
          info: adapter.info,
          async requestDevice() {
            const device = await adapter.requestDevice(); devices.push(device);
            device.addEventListener("uncapturederror", (event) => errors.push(event.error.message));
            const createTexture = device.createTexture.bind(device);
            device.createTexture = (descriptor) => {
              const texture = createTexture(descriptor); const destroy = texture.destroy.bind(texture);
              if (descriptor.label?.startsWith("VirGL resident output")) {
                created += 1; alive.add(texture);
                texture.destroy = () => { if (alive.delete(texture)) destroyed += 1; destroy(); };
              }
              return texture;
            };
            return device;
          },
        };
      },
    };
    const canvas = document.createElement("canvas"); const status = document.createElement("p");
    document.body.append(canvas, status);
    const display = new GuestDisplay(canvas, status, { navigator: { gpu } });
    const dimensions = { canvasWidth: 128, canvasHeight: 96 };
    const clear = (sequence) => virglClearPacket({ ...dimensions, sequence, version: 2 });
    const samples = [];
    try {
      for (let cycle = 0; cycle < 32; cycle += 1) {
        const sequence = 1000 + cycle * 3;
        require((await display.present3d(clear(sequence))).resident, `Clear ${cycle} was not resident`);
        const readback = display.present3d(virglResidentReadbackPacket({
          ...dimensions, producerSequence: sequence, sequence: sequence + 1,
          sourceRect: { x: 4, y: 8, width: 8, height: 4 },
        }));
        const release = display.present3d(virglResidentReleasePacket({ producerSequence: sequence }));
        const result = await readback;
        require(result.success && result.readback?.pixels.length === 128, `Queued readback ${cycle} failed`);
        const expected = result.readback.format === 1 ? [191, 128, 64, 255] : [64, 128, 191, 255];
        const pixel = [...result.readback.pixels.subarray(0, 4)];
        for (let index = 0; index < result.readback.pixels.length; index += 4) {
          require(expected.every((value, channel) => Math.abs(result.readback.pixels[index + channel] - value) <= 1),
            `Wrong GPU readback pixel at ${cycle}/${index}`);
        }
        require(Object.keys(await release).length === 0, "Release generated a guest acknowledgment");
        await display.whenIdle(); require(alive.size === 0, `Resident texture leaked after ${cycle}`);
        samples.push({ sequence, format: result.readback.format, pixel });
      }
      for (let cycle = 0; cycle < 32; cycle += 1) {
        const sequence = 2000 + cycle;
        const draw = display.present3d(clear(sequence));
        const release = display.present3d(virglResidentReleasePacket({ producerSequence: sequence }));
        require((await draw).resident, `Delayed publication ${cycle} failed`);
        await release; await display.whenIdle();
        require(alive.size === 0, `Release before publication leaked after ${cycle}`);
      }
      const old = display.present3d(clear(777));
      const stale = display.present3d(virglResidentReleasePacket({ producerSequence: 777 }));
      display.reset();
      const fresh = display.present3d(clear(777));
      require(!(await old).success, "Reset accepted an old producer"); await stale;
      require((await fresh).resident && alive.size === 1, "Stale release destroyed the new producer");
      const first = await display.acquireWebGpuBackend();
      first.device.destroy(); await first.device.lost;
      const second = await display.acquireWebGpuBackend();
      require(second.deviceGeneration === first.deviceGeneration + 1, "Device generation did not advance");
      require(alive.size === 0, "Old generation retained a resident texture");
      require((await display.present3d(clear(777))).resident, "New device could not reuse the producer sequence");
      await display.present3d(virglResidentReleasePacket({ producerSequence: 777 }));
      await second.device.queue.onSubmittedWorkDone(); await display.whenIdle();
      require(alive.size === 0 && created === destroyed, "Resident create/destroy counts differ");
      require(errors.length === 0, errors.join("; "));
      return { adapter: second.adapterInfo, cycles: samples.length, publicationReleaseCycles: 32, samples,
        created, destroyed, liveResidentTextures: alive.size,
        deviceGenerations: [first.deviceGeneration, second.deviceGeneration], devices: devices.length, errors };
    } finally {
      display.destroy(); for (const device of devices) device.destroy();
      canvas.remove(); status.remove();
    }
  });
}
