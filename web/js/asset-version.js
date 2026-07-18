export const WEBBOXVM_ASSET_VERSION = "20260718-multicore";

export function versionedUrl(path, baseUrl = import.meta.url) {
  const url = new URL(path, baseUrl);
  url.searchParams.set("v", WEBBOXVM_ASSET_VERSION);
  return url;
}
