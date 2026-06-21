const DISK_FILE_NAME = "webboxvm-install-disk.wbdisk";
const COMPRESSED_DISK_MAGIC = new Uint8Array([0x57, 0x42, 0x44, 0x5a, 0x30, 0x30, 0x30, 0x31]);

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
      if (file.size === 0) {
        return undefined;
      }
      return decodeDiskSnapshotFromStorage(new Uint8Array(await file.arrayBuffer()));
    } catch (error) {
      if (error.name !== "NotFoundError") {
        throw error;
      }
      return undefined;
    }
  }

  async write(snapshot) {
    const stored = await encodeDiskSnapshotForStorage(snapshot);
    const handle = await (await this.#getRoot()).getFileHandle(DISK_FILE_NAME, { create: true });
    const writable = await handle.createWritable();
    try {
      await writable.write(stored);
      await writable.close();
    } catch (error) {
      await writable.abort?.().catch(() => {});
      throw error;
    }
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

export async function encodeDiskSnapshotForStorage(snapshot) {
  const compressed = await compressBytes(snapshot);
  if (!compressed || compressed.byteLength + COMPRESSED_DISK_MAGIC.byteLength >= snapshot.byteLength) {
    return snapshot;
  }

  const stored = new Uint8Array(COMPRESSED_DISK_MAGIC.byteLength + compressed.byteLength);
  stored.set(COMPRESSED_DISK_MAGIC);
  stored.set(compressed, COMPRESSED_DISK_MAGIC.byteLength);
  return stored;
}

export async function decodeDiskSnapshotFromStorage(stored) {
  if (!startsWith(stored, COMPRESSED_DISK_MAGIC)) {
    return stored;
  }

  return decompressBytes(stored.subarray(COMPRESSED_DISK_MAGIC.byteLength));
}

async function compressBytes(bytes) {
  if (typeof CompressionStream !== "function") {
    return undefined;
  }
  return transformBytes(bytes, new CompressionStream("gzip"));
}

async function decompressBytes(bytes) {
  if (typeof DecompressionStream !== "function") {
    throw new Error("Compressed disk snapshot needs DecompressionStream support");
  }
  return transformBytes(bytes, new DecompressionStream("gzip"));
}

async function transformBytes(bytes, stream) {
  const output = new Response(stream.readable).arrayBuffer();
  const writer = stream.writable.getWriter();
  await writer.write(bytes);
  await writer.close();
  return new Uint8Array(await output);
}

function startsWith(bytes, prefix) {
  return prefix.every((byte, index) => bytes[index] === byte);
}
