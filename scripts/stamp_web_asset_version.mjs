#!/usr/bin/env node

import { readdir, readFile, writeFile } from "node:fs/promises";
import { dirname, extname, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const WEB_ROOT = resolve(REPOSITORY_ROOT, "web");
const GENERATED_DIRECTORIES = new Set(["pkg", "pkg-threaded"]);
const MODULE_PATTERNS = [
  /(\b(?:import|export)\s+[^;]*?\bfrom\s*)(["'`])((?:\.\.?\/)[^"'`]+)\2/g,
  /(\bimport\s*\(\s*)(["'`])((?:\.\.?\/)[^"'`]+)\2/g,
  /(\bimport\s*)(["'`])((?:\.\.?\/)[^"'`]+)\2/g,
];

export function findRelativeModuleSpecifiers(source) {
  const specifiers = [];
  rewriteModuleSpecifiers(source, (specifier) => {
    specifiers.push(specifier);
    return specifier;
  });
  return specifiers;
}

export function stampRelativeModuleSpecifiers(source, version) {
  return rewriteModuleSpecifiers(source, (specifier) => stampSpecifier(specifier, version));
}

export async function checkWebAssetGraph() {
  const version = await readAssetVersion();
  const errors = [];
  for (const file of await webModuleFiles()) {
    const source = await readFile(file, "utf8");
    for (const specifier of findRelativeModuleSpecifiers(source)) {
      const versions = queryValues(specifier, "v");
      if (versions.length !== 1 || versions[0] !== version) {
        errors.push(`${relative(REPOSITORY_ROOT, file)}: ${specifier}`);
      }
    }
  }
  const html = await readFile(resolve(WEB_ROOT, "index.html"), "utf8");
  if (!html.includes(`src="./app.js?v=${version}"`)) {
    errors.push("web/index.html: module entrypoint is not stamped");
  }
  return errors;
}

export async function stampWebAssetGraph() {
  const version = await readAssetVersion();
  let changed = 0;
  for (const file of await webModuleFiles()) {
    const source = await readFile(file, "utf8");
    const stamped = stampRelativeModuleSpecifiers(source, version);
    if (stamped !== source) {
      await writeFile(file, stamped);
      changed += 1;
    }
  }
  const indexPath = resolve(WEB_ROOT, "index.html");
  const html = await readFile(indexPath, "utf8");
  const stampedHtml = html.replace(
    /src="\.\/app\.js(?:\?[^"]*)?"/,
    `src="./app.js?v=${version}"`,
  );
  if (stampedHtml !== html) {
    await writeFile(indexPath, stampedHtml);
    changed += 1;
  }
  return changed;
}

function rewriteModuleSpecifiers(source, rewrite) {
  for (const pattern of MODULE_PATTERNS) {
    source = source.replace(pattern, (match, prefix, quote, specifier) => {
      return `${prefix}${quote}${rewrite(specifier)}${quote}`;
    });
  }
  return source;
}

function stampSpecifier(specifier, version) {
  const hashIndex = specifier.indexOf("#");
  const hash = hashIndex === -1 ? "" : specifier.slice(hashIndex);
  const withoutHash = hashIndex === -1 ? specifier : specifier.slice(0, hashIndex);
  const queryIndex = withoutHash.indexOf("?");
  const pathname = queryIndex === -1 ? withoutHash : withoutHash.slice(0, queryIndex);
  const query = queryIndex === -1 ? "" : withoutHash.slice(queryIndex + 1);
  const rest = query
    .split("&")
    .filter((part) => part && part.split("=", 1)[0] !== "v");
  return `${pathname}?v=${version}${rest.length ? `&${rest.join("&")}` : ""}${hash}`;
}

function queryValues(specifier, name) {
  const query = specifier.split("#", 1)[0].split("?", 2)[1] ?? "";
  return query
    .split("&")
    .filter((part) => part.split("=", 1)[0] === name)
    .map((part) => part.slice(name.length + 1));
}

async function readAssetVersion() {
  const source = await readFile(resolve(WEB_ROOT, "js/asset-version.js"), "utf8");
  const match = source.match(/WEBBOXVM_ASSET_VERSION\s*=\s*"([^"]+)"/);
  if (!match) throw new Error("WEBBOXVM_ASSET_VERSION was not found");
  return match[1];
}

async function webModuleFiles(directory = WEB_ROOT) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && GENERATED_DIRECTORIES.has(entry.name)) continue;
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await webModuleFiles(path)));
    if (entry.isFile() && [".js", ".mjs"].includes(extname(entry.name))) files.push(path);
  }
  return files.sort();
}

async function main() {
  const mode = process.argv[2];
  if (mode === "--write") {
    console.log(`Stamped ${await stampWebAssetGraph()} file(s)`);
    return;
  }
  if (mode === "--check") {
    const errors = await checkWebAssetGraph();
    if (errors.length) throw new Error(`Unversioned module edges:\n${errors.join("\n")}`);
    console.log("Web asset module graph is consistently versioned");
    return;
  }
  throw new Error("usage: stamp_web_asset_version.mjs --write|--check");
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) await main();
