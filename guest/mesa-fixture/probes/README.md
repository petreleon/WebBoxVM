# Stock Mesa startup probes

These clients use the unmodified F02 Mesa 25.3.6 ABI and a real `virtio_gpu`
DRM character device. Compile them in the pinned, offline ARM64 builder after
the Mesa build has produced its manifest. `compile.py` records the exact compiler
command, source hashes, ELF metadata and binary hashes in a fresh output directory.

The image installs `/usr/bin/webboxvm-mesa-gles` and
`/usr/bin/webboxvm-mesa-vulkan`. Each accepts one optional DRM device path;
the default is `/dev/dri/renderD128`. Output includes actual API strings and
the first failed stage. The process exits nonzero on unsupported requests.

The GLES probe requires the GBM EGL platform, a real GLES 3 context,
`EGL_MESA_query_driver` identifying `virtio_gpu`, and the Mesa 25.3.6 version
string. VirGL can expose its host renderer name, so the probe checks the loaded
driver independently and rejects software renderer strings.

The Vulkan probe requires Vulkan 1.2 properties queries, the Mesa Venus driver
ID, its `venus` name and Mesa 25.3.6 version, a non-CPU physical device, and
successful graphics device/queue creation and idle wait.

Exact success lines, emitted after cleanup:

- `I01_MESA_GLES_STARTUP_PASS`
- `I01_MESA_VULKAN_STARTUP_PASS`

These are startup checks. They do not certify rendered pixels, OpenGL 4.6,
GLES 3.2, Vulkan 1.4, conformance, real applications or performance. No probe
changes caps, enables software fallback, or generates host GPU completions.
