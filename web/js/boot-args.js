const INSTALLED_DISK_BENCHMARK = "installed-disk";

export function extraBootargsFromLocation(locationLike = globalThis.location) {
  if (!locationLike?.href) {
    return "";
  }
  const params = new URL(locationLike.href).searchParams;
  return normalizeExtraBootargs(params.get("bootargs"));
}

export function normalizeExtraBootargs(value) {
  return String(value ?? "").trim().split(/\s+/).filter(Boolean).join(" ");
}

export function installedDiskBenchmarkFromLocation(locationLike = globalThis.location) {
  if (!locationLike?.href) {
    return false;
  }
  const params = new URL(locationLike.href).searchParams;
  return params.get("benchmark") === INSTALLED_DISK_BENCHMARK;
}

export function stagedSmpRequestedFromLocation(locationLike = globalThis.location) {
  if (!locationLike?.href) {
    return true;
  }
  const value = new URL(locationLike.href).searchParams.get("staged-smp");
  if (value === null || value === "on") {
    return true;
  }
  if (value === "off") {
    return false;
  }
  throw new Error("staged-smp must be 'on' or 'off'");
}
