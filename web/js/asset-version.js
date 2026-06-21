export const WEBBOXVM_ASSET_VERSION = "20260621-ext4-hook-dirs";

export function versionedUrl(path, baseUrl = import.meta.url) {
  const url = new URL(path, baseUrl);
  url.searchParams.set("v", WEBBOXVM_ASSET_VERSION);
  return url;
}
