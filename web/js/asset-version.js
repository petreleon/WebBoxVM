// After changing this token, run: node scripts/stamp_web_asset_version.mjs --write
export const WEBBOXVM_ASSET_VERSION = "20260904-virgl-depth-batch-r1";

export function versionedUrl(path, baseUrl = import.meta.url) {
  const url = new URL(path, baseUrl);
  url.searchParams.set("v", WEBBOXVM_ASSET_VERSION);
  return url;
}
