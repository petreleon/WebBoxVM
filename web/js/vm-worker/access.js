let accessTail = Promise.resolve();

export async function withEmulatorAccess(action) {
  const previous = accessTail;
  let release;
  accessTail = new Promise((resolve) => {
    release = resolve;
  });
  await previous.catch(() => {});
  try {
    return await action();
  } finally {
    release();
  }
}
