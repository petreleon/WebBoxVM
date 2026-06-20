# WebBoxVM — Active Todo

History: [sprint-history.md](sprint-history.md).

## Now — Browser Install Path
- [x] Prove browser networking end to end: VirtIO-net, DHCP, DNS, and Debian package download through WebSocket NAT
- [ ] Continue Debian installer from the next real blocker after routed networking
- [ ] Profile installer/userland hot paths only when they block forward progress

## Next Product Milestone — Boot from Disk
- [ ] Add `Boot from disk` from the persisted OPFS virtual disk
- [ ] Keep `Boot ISO` for installer/media boot
- [ ] Make disk boot mean: no ISO kernel/initrd handoff, no installer restart
- [ ] Test saved disk restore plus primary disk boot-source selection

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Done
- [x] ISO boot + OPFS disk + WS NAT.
