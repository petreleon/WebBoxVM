export const GIB = 1024n ** 3n;

export function formatBytes(bytes) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KiB", "MiB", "GiB"];
  let value = bytes / 1024;
  let unit = units.shift();
  while (value >= 1024 && units.length > 0) {
    value /= 1024;
    unit = units.shift();
  }
  return `${value.toFixed(value >= 10 ? 1 : 2)} ${unit}`;
}

export function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

export function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function nextFrame() {
  return new Promise((resolve) => {
    let done = false;
    const finish = () => {
      if (!done) {
        done = true;
        resolve();
      }
    };
    requestAnimationFrame(finish);
    setTimeout(finish, 0);
  });
}
