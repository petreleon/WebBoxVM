export function bootPreparedInstalledDisk(
  EmulatorClass,
  diskSnapshot,
  extraBootargs,
  preparation,
  onCreated = () => {},
) {
  const emulator = new EmulatorClass(preparation.bootCores);
  onCreated(emulator);
  const result = emulator.boot_installed_disk_with_staged_smp(
    diskSnapshot,
    preparation.bootCores,
    extraBootargs,
    preparation.parallelReady,
  );
  return { emulator, result };
}
