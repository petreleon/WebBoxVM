const DISK_FILE_NAME = "webboxvm-install-disk.wbdisk";

export class OpfsDiskStore {
  #root;

  static available() {
    return Boolean(navigator.storage?.getDirectory);
  }

  async requestPersistence() {
    if (navigator.storage.persist) {
      await navigator.storage.persist();
    }
  }

  async size() {
    try {
      const handle = await (await this.#getRoot()).getFileHandle(DISK_FILE_NAME);
      return (await handle.getFile()).size;
    } catch (error) {
      if (error.name !== "NotFoundError") {
        throw error;
      }
      return 0;
    }
  }

  async load() {
    try {
      const handle = await (await this.#getRoot()).getFileHandle(DISK_FILE_NAME);
      const file = await handle.getFile();
      return file.size === 0 ? undefined : new Uint8Array(await file.arrayBuffer());
    } catch (error) {
      if (error.name !== "NotFoundError") {
        throw error;
      }
      return undefined;
    }
  }

  async write(snapshot) {
    const handle = await (await this.#getRoot()).getFileHandle(DISK_FILE_NAME, { create: true });
    const writable = await handle.createWritable();
    await writable.write(snapshot);
    await writable.close();
  }

  async clear() {
    try {
      await (await this.#getRoot()).removeEntry(DISK_FILE_NAME);
    } catch (error) {
      if (error.name !== "NotFoundError") {
        throw error;
      }
    }
  }

  async #getRoot() {
    this.#root ??= await navigator.storage.getDirectory();
    return this.#root;
  }
}
