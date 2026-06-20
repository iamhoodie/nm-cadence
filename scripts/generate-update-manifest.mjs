import { readdir, readFile, writeFile, mkdir } from "node:fs/promises";
import path from "node:path";

function argValue(flag) {
  const index = process.argv.indexOf(flag);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

const baseUrl = argValue("--base-url") || process.env.UPDATE_BASE_URL;
const target = argValue("--target") || process.env.UPDATE_TARGET;
const arch = argValue("--arch") || process.env.UPDATE_ARCH;
const notes = argValue("--notes") || process.env.UPDATE_NOTES || "";

if (!baseUrl || !target || !arch) {
  console.error("Usage: node scripts/generate-update-manifest.mjs --base-url <url> --target <target> --arch <arch> [--notes \"...\"]");
  process.exit(1);
}

const repoRoot = process.cwd();
const tauriConfigPath = path.join(repoRoot, "src-tauri", "tauri.conf.json");
const bundleDir = path.join(repoRoot, "src-tauri", "target", "release", "bundle");
const manifestDir = path.join(repoRoot, "release-manifests", target, arch);

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) return walk(fullPath);
      return [fullPath];
    })
  );
  return files.flat();
}

const config = JSON.parse(await readFile(tauriConfigPath, "utf8"));
const version = config.version;
const sigFiles = (await walk(bundleDir)).filter((file) => file.endsWith(".sig"));

if (sigFiles.length === 0) {
  console.error(`No updater signature files found under ${bundleDir}. Run a signed Tauri release build first.`);
  process.exit(1);
}

if (sigFiles.length > 1) {
  console.error("Found multiple updater signature files. Narrow the release output before generating a manifest:");
  for (const file of sigFiles) console.error(`- ${path.relative(repoRoot, file)}`);
  process.exit(1);
}

const sigPath = sigFiles[0];
const artifactPath = sigPath.slice(0, -4);
const artifactName = path.basename(artifactPath);
const signature = (await readFile(sigPath, "utf8")).trim();

const manifest = {
  version,
  pub_date: new Date().toISOString(),
  url: `${baseUrl.replace(/\/$/, "")}/${artifactName}`,
  signature,
  notes,
};

await mkdir(manifestDir, { recursive: true });
const manifestPath = path.join(manifestDir, `${version}.json`);
await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

console.log(`Wrote ${path.relative(repoRoot, manifestPath)}`);
