import { execSync } from "node:child_process";

export const repo = "https://github.com/germolinal/simple";

/**
 * The release this site describes: $SIMPLE_VERSION (set by the release
 * workflow to the tag being published) or, when building locally, the latest
 * `v*` tag. Undefined if there is neither.
 */
function findVersion(): string | undefined {
  const fromEnv = process.env.SIMPLE_VERSION?.trim();
  if (fromEnv) return fromEnv;
  try {
    const tag = execSync("git describe --tags --abbrev=0 --match 'v*'", {
      stdio: ["ignore", "pipe", "ignore"],
    })
      .toString()
      .trim();
    return tag || undefined;
  } catch {
    return undefined;
  }
}

export const version = findVersion();

export interface Platform {
  name: string;
  detail: string;
  target: string;
  ext: "tar.gz" | "zip";
}

// Must match the matrix in .github/workflows/release.yaml
export const platforms: Platform[] = [
  { name: "macOS", detail: "Apple silicon", target: "aarch64-apple-darwin", ext: "tar.gz" },
  { name: "Linux", detail: "x86-64", target: "x86_64-unknown-linux-gnu", ext: "tar.gz" },
  { name: "Windows", detail: "x86-64", target: "x86_64-pc-windows-msvc", ext: "zip" },
];

// Must match the binaries packaged in .github/workflows/release.yaml
export const binaries = ["simple", "spict", "sfluxmtx", "sfalsecolor", "scompare"];

export function assetUrl(v: string, p: Platform): string {
  return `${repo}/releases/download/${v}/simple-${v}-${p.target}.${p.ext}`;
}

export const releasesUrl = `${repo}/releases`;
