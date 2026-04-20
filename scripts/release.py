#!/usr/bin/env python3
"""
Peppi release builder.

Bumps the version, builds a signed Tauri MSI, uploads the installer + updater
manifest to Cloudflare R2, and tags the git release.

Usage:
    python scripts/release.py --env-file .env.release --patch
    python scripts/release.py --env-file ~/secrets/peppi.env --minor --notes "..."
    python scripts/release.py --env-file .env.release --version 1.2.0
    python scripts/release.py --env-file .env.release --patch --skip-upload --skip-git

Required: --env-file PATH
    Points to a dotenv-format file holding the release secrets. The file must
    exist. Keys defined there overwrite the current shell environment for this
    run so you always know exactly what secrets the release is using.

    Required keys inside the env file:
        TAURI_SIGNING_PRIVATE_KEY_PASSWORD  signing key password
        R2_ACCOUNT_ID                       Cloudflare account ID
        R2_ACCESS_KEY_ID                    R2 API token access key
        R2_SECRET_ACCESS_KEY                R2 API token secret
        R2_BUCKET                           bucket name (e.g. "peppi-releases")

    Example file:
        # peppi release secrets — keep out of git
        TAURI_SIGNING_PRIVATE_KEY_PASSWORD=hunter2
        R2_ACCOUNT_ID=abcdef...
        R2_ACCESS_KEY_ID=...
        R2_SECRET_ACCESS_KEY=...
        R2_BUCKET=peppi-releases

Dependencies:
    pip install -r scripts/requirements.txt   (just boto3)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

try:
    import boto3
    from botocore.config import Config as BotoConfig
except ImportError:
    print("Missing dependency. Run: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

PROJECT_ROOT = Path(__file__).resolve().parent.parent
TAURI_CONF = PROJECT_ROOT / "src-tauri" / "tauri.conf.json"
PACKAGE_JSON = PROJECT_ROOT / "package.json"
LATEST_JSON = PROJECT_ROOT / "latest.json"
BUNDLE_DIR = PROJECT_ROOT / "src-tauri" / "target" / "release" / "bundle" / "msi"

# R2 public base — must match the updater endpoint in tauri.conf.json
R2_PUBLIC_BASE = "https://pub-a7a1511ed0f84ebbb1afa93d4fe41cb6.r2.dev"

# R2 object keys
MSI_KEY_PREFIX = "msi/"
LATEST_JSON_KEY = "latest.json"

# Default signing key path (override with --key-path)
DEFAULT_KEY_PATH = Path.home() / "peppi-keys" / "peppi.key"

# Env var names
ENV_SIGNING_PASSWORD = "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
ENV_R2_ACCOUNT = "R2_ACCOUNT_ID"
ENV_R2_ACCESS_KEY = "R2_ACCESS_KEY_ID"
ENV_R2_SECRET = "R2_SECRET_ACCESS_KEY"
ENV_R2_BUCKET = "R2_BUCKET"


# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

def step(msg: str) -> None:
    print(f"\n=== {msg} ===", flush=True)


def info(msg: str) -> None:
    print(f"  {msg}", flush=True)


def ok(msg: str) -> None:
    print(f"  [OK] {msg}", flush=True)


def warn(msg: str) -> None:
    print(f"  [!] {msg}", flush=True)


def die(msg: str) -> None:
    print(f"\nERROR: {msg}", file=sys.stderr, flush=True)
    sys.exit(1)


# ---------------------------------------------------------------------------
# Env file loading
# ---------------------------------------------------------------------------

def load_env_file(path: Path) -> None:
    """
    Parse a dotenv-format file and set the values into os.environ,
    overwriting any existing values for the same keys. Supports:

      KEY=value
      KEY="quoted value"
      KEY='single-quoted'
      export KEY=value          # leading `export` is ignored
      # comments and blank lines

    No interpolation, no multi-line values — keep release secrets simple.
    """
    if not path.exists():
        die(f"Env file not found: {path}")
    if not path.is_file():
        die(f"Env file is not a regular file: {path}")

    loaded = 0
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export "):].lstrip()

        if "=" not in line:
            die(f"{path}:{line_no}: expected KEY=VALUE, got: {raw}")

        key, _, value = line.partition("=")
        key = key.strip()
        value = value.strip()

        if not key or not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", key):
            die(f"{path}:{line_no}: invalid key name: {key!r}")

        # Strip matching surrounding quotes
        if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
            value = value[1:-1]

        os.environ[key] = value
        loaded += 1

    info(f"Loaded {loaded} key(s) from {path}")


def require_env(names: list[str], context: str) -> None:
    """Die if any required env var is missing or empty."""
    missing = [n for n in names if not os.environ.get(n)]
    if missing:
        die(f"{context} requires env var(s) missing from env file: {', '.join(missing)}")


# ---------------------------------------------------------------------------
# Versioning
# ---------------------------------------------------------------------------

VERSION_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def read_current_version() -> str:
    return json.loads(TAURI_CONF.read_text(encoding="utf-8"))["version"]


def parse_version(v: str) -> tuple[int, int, int]:
    m = VERSION_RE.match(v)
    if not m:
        die(f"Invalid version '{v}'. Expected X.Y.Z")
    return int(m.group(1)), int(m.group(2)), int(m.group(3))


def bump_version(current: str, part: str) -> str:
    major, minor, patch = parse_version(current)
    if part == "patch":
        patch += 1
    elif part == "minor":
        minor += 1
        patch = 0
    elif part == "major":
        major += 1
        minor = 0
        patch = 0
    else:
        die(f"Unknown bump part: {part}")
    return f"{major}.{minor}.{patch}"


def write_json_version(path: Path, version: str) -> None:
    text = path.read_text(encoding="utf-8")
    new_text, count = re.subn(
        r'("version"\s*:\s*")[^"]*(")',
        rf'\g<1>{version}\g<2>',
        text,
        count=1,
    )
    if count == 0:
        die(f"Could not find version field in {path}")
    path.write_text(new_text, encoding="utf-8")


# ---------------------------------------------------------------------------
# Build + sign
# ---------------------------------------------------------------------------

def run_build(key_path: Path, password: str, dry_run: bool) -> None:
    env = os.environ.copy()
    env["TAURI_SIGNING_PRIVATE_KEY"] = key_path.read_text(encoding="utf-8")
    env[ENV_SIGNING_PASSWORD] = password

    cmd = ["bun", "run", "tauri", "build", "--", "--features", "real-recording"]
    info("Running: " + " ".join(cmd))
    if dry_run:
        return

    # shell=True on Windows so `bun` resolves from PATH via cmd.exe
    result = subprocess.run(cmd, cwd=PROJECT_ROOT, env=env, shell=True)
    if result.returncode != 0:
        die(f"Build failed with exit code {result.returncode}")


def zip_msi(src: Path, dst: Path) -> None:
    """Create the updater archive (zipped MSI)."""
    if dst.exists():
        dst.unlink()
    with zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.write(src, arcname=src.name)


def ensure_signature(zip_path: Path, key_path: Path, password: str, dry_run: bool) -> str:
    """
    Return the updater signature.

    With `createUpdaterArtifacts: true` set in tauri.conf.json, Tauri already
    produced `<msi>.zip.sig` alongside the MSI during the build. Prefer that.
    Fall back to invoking `tauri signer sign` manually if missing (e.g. when
    called with --skip-build after a plain dev build).
    """
    if dry_run:
        return "DRY_RUN_SIGNATURE"

    sig_path = Path(f"{zip_path}.sig")
    if sig_path.exists():
        info(f"Using existing signature: {sig_path.name}")
        return sig_path.read_text(encoding="utf-8").strip()

    info("Signing manually (no auto-generated .sig found)...")
    private_key = key_path.read_text(encoding="utf-8")
    result = subprocess.run(
        [
            "bunx", "tauri", "signer", "sign",
            "--private-key", private_key,
            "--password", password,
            str(zip_path),
        ],
        capture_output=True, text=True, shell=True, cwd=PROJECT_ROOT,
    )
    if result.returncode != 0:
        die(f"Signing failed: {result.stderr.strip()}")

    if sig_path.exists():
        return sig_path.read_text(encoding="utf-8").strip()

    # Last resort: scrape stdout
    match = re.search(r"^dW50cnVzdGVk.*$", result.stdout, re.MULTILINE)
    if match:
        return match.group(0)
    die("Could not extract signature from signing output")


# ---------------------------------------------------------------------------
# R2 upload
# ---------------------------------------------------------------------------

def make_r2_client(account_id: str, access_key: str, secret_key: str):
    endpoint = f"https://{account_id}.r2.cloudflarestorage.com"
    return boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_key,
        region_name="auto",
        config=BotoConfig(signature_version="s3v4"),
    )


def upload_to_r2(client, bucket: str, key: str, file_path: Path, content_type: str) -> None:
    info(f"Uploading {file_path.name} -> s3://{bucket}/{key}")
    client.upload_file(
        str(file_path),
        bucket,
        key,
        ExtraArgs={
            "ContentType": content_type,
            # Short cache so updater sees new releases quickly
            "CacheControl": "public, max-age=60",
        },
    )


# ---------------------------------------------------------------------------
# Git
# ---------------------------------------------------------------------------

def git(args: list[str], dry_run: bool) -> None:
    info("git " + " ".join(args))
    if dry_run:
        return
    result = subprocess.run(["git", *args], cwd=PROJECT_ROOT)
    if result.returncode != 0:
        die(f"git {' '.join(args)} failed")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(
        description="Peppi release builder",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    ap.add_argument(
        "--env-file",
        type=Path,
        required=True,
        metavar="PATH",
        help="Path to dotenv file holding release secrets (required). See docstring for required keys.",
    )

    bump = ap.add_mutually_exclusive_group()
    bump.add_argument("--patch", action="store_const", dest="bump", const="patch", help="Bump patch (X.Y.Z+1)")
    bump.add_argument("--minor", action="store_const", dest="bump", const="minor", help="Bump minor (X.Y+1.0)")
    bump.add_argument("--major", action="store_const", dest="bump", const="major", help="Bump major (X+1.0.0)")
    bump.add_argument("--version", help="Explicit version X.Y.Z (overrides bump flags)")

    ap.add_argument("--notes", default="", help="Release notes")
    ap.add_argument("--key-path", type=Path, default=DEFAULT_KEY_PATH, help="Tauri signing private key file")
    ap.add_argument("--skip-build", action="store_true", help="Don't rebuild; reuse existing MSI")
    ap.add_argument("--skip-upload", action="store_true", help="Skip R2 upload")
    ap.add_argument("--skip-git", action="store_true", help="Skip git commit/tag")
    ap.add_argument("--dry-run", action="store_true", help="Print actions without changing anything")
    return ap.parse_args()


def resolve_new_version(args: argparse.Namespace, current: str) -> str:
    if args.version:
        parse_version(args.version)
        return args.version
    if args.bump:
        return bump_version(current, args.bump)
    prompted = input("New version (or rerun with --patch/--minor/--major): ").strip()
    parse_version(prompted)
    return prompted


def load_r2_env(skip_upload: bool):
    """Return (client, bucket) or (None, None) if skipping upload."""
    if skip_upload:
        return None, None

    require_env(
        [ENV_R2_ACCOUNT, ENV_R2_ACCESS_KEY, ENV_R2_SECRET, ENV_R2_BUCKET],
        "R2 upload",
    )

    client = make_r2_client(
        os.environ[ENV_R2_ACCOUNT],
        os.environ[ENV_R2_ACCESS_KEY],
        os.environ[ENV_R2_SECRET],
    )
    return client, os.environ[ENV_R2_BUCKET]


def main() -> None:
    args = parse_args()

    # Load env file FIRST so every downstream env lookup sees its values.
    load_env_file(args.env_file)

    current = read_current_version()
    info(f"Current version: {current}")

    new_version = resolve_new_version(args, current)
    if new_version == current:
        die(f"Version {new_version} matches current; nothing to do")
    info(f"New version: {new_version}")

    if args.dry_run:
        warn("DRY RUN — nothing will actually be changed")

    # Validate signing key / password up front (before the long build runs)
    if not args.skip_build:
        if not args.key_path.exists():
            die(f"Signing key not found at {args.key_path}")
        require_env([ENV_SIGNING_PASSWORD], "Signing")

    # Validate R2 creds up front
    r2_client, r2_bucket = load_r2_env(args.skip_upload)

    notes = args.notes or input("Release notes: ").strip() or "Bug fixes and improvements"

    # --- 1. Bump version in project files ---
    step("Updating version in project files")
    if not args.dry_run:
        write_json_version(TAURI_CONF, new_version)
        write_json_version(PACKAGE_JSON, new_version)
    ok("tauri.conf.json + package.json")

    # --- 2. Build ---
    if args.skip_build:
        warn("Skipping build (--skip-build)")
    else:
        step("Building signed Tauri release")
        run_build(args.key_path, os.environ[ENV_SIGNING_PASSWORD], args.dry_run)
        ok("build complete")

    # --- 3. Locate artifacts ---
    step("Locating MSI artifact")
    msi_name = f"Peppi_{new_version}_x64_en-US.msi"
    msi_path = BUNDLE_DIR / msi_name
    zip_path = BUNDLE_DIR / f"{msi_name}.zip"

    if not args.dry_run and not msi_path.exists():
        info(f"Looking in {BUNDLE_DIR}")
        for f in sorted(BUNDLE_DIR.glob("*.msi")):
            info(f"  found: {f.name}")
        die(f"Expected MSI not found: {msi_path}")
    ok(f"MSI: {msi_name}")

    # --- 4. Zip (if Tauri didn't already) ---
    if not args.dry_run and not zip_path.exists():
        step("Creating update ZIP")
        zip_msi(msi_path, zip_path)
        ok(f"ZIP: {zip_path.name}")
    else:
        info(f"ZIP already present: {zip_path.name}")

    # --- 5. Signature ---
    step("Reading update signature")
    signature = ensure_signature(
        zip_path,
        args.key_path,
        os.environ.get(ENV_SIGNING_PASSWORD, ""),
        args.dry_run,
    )
    ok("signature ready")

    # --- 6. Write latest.json ---
    step("Writing latest.json")
    pub_date = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    download_url = f"{R2_PUBLIC_BASE}/{MSI_KEY_PREFIX}{msi_name}.zip"
    manifest = {
        "version": new_version,
        "notes": notes,
        "pub_date": pub_date,
        "platforms": {
            "windows-x86_64": {
                "signature": signature,
                "url": download_url,
            }
        },
    }
    if not args.dry_run:
        LATEST_JSON.write_text(json.dumps(manifest, separators=(",", ":")), encoding="utf-8")
    ok(f"{LATEST_JSON.name} updated")

    # --- 7. Upload to R2 ---
    if args.skip_upload:
        warn("Skipping R2 upload (--skip-upload)")
    else:
        step("Uploading to R2")
        if not args.dry_run:
            upload_to_r2(
                r2_client, r2_bucket,
                f"{MSI_KEY_PREFIX}{msi_name}.zip", zip_path, "application/zip",
            )
            upload_to_r2(
                r2_client, r2_bucket,
                LATEST_JSON_KEY, LATEST_JSON, "application/json",
            )
        ok("R2 upload complete")

    # --- 8. Git commit + tag ---
    if args.skip_git:
        warn("Skipping git commit/tag (--skip-git)")
    else:
        step("Committing and tagging")
        git(["add", "src-tauri/tauri.conf.json", "package.json", "latest.json"], args.dry_run)
        git(["commit", "-m", f"Release v{new_version}"], args.dry_run)
        git(["tag", f"v{new_version}"], args.dry_run)
        info("Remember to: git push && git push --tags")

    # --- Summary ---
    print()
    print("=" * 40)
    print(f"  Release v{new_version} complete")
    print("=" * 40)
    print(f"  Download: {download_url}")
    print(f"  Manifest: {R2_PUBLIC_BASE}/{LATEST_JSON_KEY}")
    print()


if __name__ == "__main__":
    main()
