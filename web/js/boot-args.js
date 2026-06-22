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
