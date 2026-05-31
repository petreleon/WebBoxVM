import { formatBytes, nextFrame } from "./utils.js";

export async function readSelectedIso(els, ui) {
  const file = els.isoFile.files?.[0];
  if (!file) {
    ui.setStatus("No ISO selected", "warn");
    ui.log("No ISO selected");
    return undefined;
  }

  ui.setStatus(`Reading ${file.name}`);
  ui.log(`Reading ${file.name} (${formatBytes(file.size)})`);
  await nextFrame();

  const buffer = await file.arrayBuffer();
  return {
    bytes: new Uint8Array(buffer),
    name: file.name,
  };
}

export async function fetchBundledDebian(ui) {
  const url = "./media/debian-arm64-netinst.iso";
  ui.setStatus("Fetching Debian ISO");
  ui.log(`Fetching ${url}`);
  await nextFrame();

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Debian ISO fetch failed: HTTP ${response.status}`);
  }
  const buffer = await response.arrayBuffer();
  return {
    bytes: new Uint8Array(buffer),
    name: "Debian arm64 netinst",
  };
}
