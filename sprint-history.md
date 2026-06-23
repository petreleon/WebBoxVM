# WebBoxVM — Sprint History

Done: WS NAT install -> OPFS disk boot -> Debian ttyAMA0 login.

Current: speed + ARM64/JIT semantics.

Current clean-main rebaseline: Debian login 497.9s, JIT 0 rejects/skips/fallbacks.

JIT block shape sample: login 488.1s, 399 cached blocks: 1=117, 2-4=163, 5-8=55, 9-16=28, 17-32=12, 33-64=24.

Kept speed win: skip one-instruction JIT blocks, login 483.5s, 160 cached blocks, 46 skipped blocks.

Archived slower speed trials: metrics 500ms, frame 48ms, JIT threshold 1, JIT threshold 3, VirtIO scratch buffer, step slice 10M, JIT block cap 96, scalar JIT sync, IRQ preflight, frame batches 256, autosave poll 10s, UART flush 250ms, network TX poll 64ms.
