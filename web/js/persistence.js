import { OpfsDiskStore } from "./persistence-store.js?v=20260720-input-latency-r4";
import { formatBytes } from "./utils.js?v=20260720-input-latency-r4";

const AUTOSAVE_INTERVAL_MS = 600_000;

export class DiskPersistence {
  available = false;
  persistedBytes = 0;
  saving = false;

  #autosaveIntervalMs;
  #lastSavedGeneration = 0n;
  #lastAutosaveAt = 0;
  #now;
  #saveQueued = false;
  #autosaveSuspended = false;
  #store;

  constructor({
    autosaveIntervalMs = AUTOSAVE_INTERVAL_MS,
    now = () => performance.now(),
    store = new OpfsDiskStore(),
  } = {}) {
    this.#autosaveIntervalMs = autosaveIntervalMs;
    this.#now = now;
    this.#store = store;
  }

  async init(log) {
    this.available = OpfsDiskStore.available();
    if (!this.available) {
      log("Persistent disk storage unavailable");
      return;
    }
    await this.#store.requestPersistence();
    await this.refreshInfo();
    log(
      this.persistedBytes > 0
        ? `Persistent disk ready (${formatBytes(this.persistedBytes)})`
        : "Persistent disk ready",
    );
  }

  async restoreIfPresent(emulator) {
    const snapshot = await this.load();
    if (!snapshot) {
      return "";
    }

    const result = await emulator.restore_install_disk(snapshot);
    if (result.startsWith("ERR:")) {
      throw new Error(result);
    }
    this.#lastSavedGeneration = emulator.install_disk_generation();
    return `${result} from ${formatBytes(snapshot.byteLength)} OPFS snapshot`;
  }

  markClean(emulator) {
    this.#lastSavedGeneration = emulator.install_disk_generation();
  }

  shouldAutosave(emulator) {
    if (
      !this.available ||
      this.saving ||
      this.#autosaveSuspended ||
      emulator.install_disk_generation() === this.#lastSavedGeneration
    ) {
      return false;
    }
    const now = this.#now();
    if (now - this.#lastAutosaveAt < this.#autosaveIntervalMs) {
      return false;
    }
    this.#lastAutosaveAt = now;
    return true;
  }

  async save(emulator, { force = false, quiet = false, log = () => {}, after = () => {} } = {}) {
    if (!emulator || !this.available) {
      return;
    }

    const generation = emulator.install_disk_generation();
    if (!force && generation === this.#lastSavedGeneration) {
      return;
    }
    if (this.saving) {
      this.#saveQueued = true;
      return;
    }

    this.saving = true;
    after();
    try {
      await this.#writeSnapshot(emulator, generation, quiet, log);
    } catch (error) {
      if (!force && isQuotaExceededError(error)) {
        this.#autosaveSuspended = true;
        this.#saveQueued = false;
        await this.refreshInfo().catch(() => {});
        log(`Autosave paused: storage quota reached (${formatBytes(this.persistedBytes)} saved)`);
        return;
      }
      throw error;
    } finally {
      this.saving = false;
      after();
      await this.#drainQueued(emulator, log, after);
    }
  }

  async clear(emulator, log) {
    if (!this.available) {
      return;
    }
    await this.#store.clear();
    this.persistedBytes = 0;
    this.#autosaveSuspended = false;
    this.#lastSavedGeneration = emulator ? emulator.install_disk_generation() : 0n;
    log("Cleared saved disk");
  }

  async refreshInfo() {
    this.persistedBytes = this.available ? await this.#store.size() : 0;
  }

  async load() {
    if (!this.available) {
      return undefined;
    }
    const bytes = await this.#store.load();
    this.persistedBytes = bytes?.byteLength ?? 0;
    return bytes;
  }

  async #writeSnapshot(emulator, generation, quiet, log) {
    const snapshot = await emulator.install_disk_snapshot();
    await this.#store.write(snapshot);
    this.persistedBytes = snapshot.byteLength;
    this.#lastSavedGeneration = generation;
    if (!quiet) {
      log(`Saved disk (${formatBytes(snapshot.byteLength)})`);
    }
  }

  async #drainQueued(emulator, log, after) {
    if (!this.#saveQueued || this.#autosaveSuspended) {
      return;
    }
    this.#saveQueued = false;
    await this.save(emulator, { quiet: true, log, after });
  }
}

function isQuotaExceededError(error) {
  return error?.name === "QuotaExceededError";
}
