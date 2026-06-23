export const WEBBOXVM_ASSET_VERSION = "20260623-fast-slice";

export function versionedUrl(path, baseUrl = import.meta.url) {
  const url = new URL(path, baseUrl);
  url.searchParams.set("v", WEBBOXVM_ASSET_VERSION);
  return url;
}
