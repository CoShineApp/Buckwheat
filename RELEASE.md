# How to release Peppi

Do this from the project root. Replace `1.0.2` with your new version everywhere.

---

## 1. Bump version

Edit these two files and set `"version": "1.0.2"` (same value in both):

- `src-tauri/tauri.conf.json` — change the `"version"` field
- `package.json` — change the `"version"` field

---

## 2. Build (with signing + real-recording)

Release builds **must** use the `real-recording` feature (actual screen capture). Set your signing key, then build.

**Option A – release script (easiest)**

Key at default path `C:\Users\cafebabe\peppi-keys\peppi.key`:

```powershell
.\scripts\release.ps1 -Version "1.0.2"
```

Custom key path:

```powershell
.\scripts\release.ps1 -Version "1.0.2" -KeyPath "C:\path\to\peppi.key"
```

The script bumps version, builds with `--features real-recording`, zips/signs the MSI, and creates `latest.json`.

**Option B – manual build**

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "C:\path\to\peppi.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your-key-password"
bun run tauri build -- --features real-recording
```

Then do steps 3–5 below (zip, sign, create latest.json) yourself.

---

## 3. Zip and sign the MSI for the updater

The MSI will be at:

`src-tauri\target\release\bundle\msi\Peppi_1.0.2_x64_en-US.msi`

**Zip it**

- Right‑click the MSI → Send to → Compressed (zipped) folder  
- Or in PowerShell: `Compress-Archive -Path "src-tauri\target\release\bundle\msi\Peppi_1.0.2_x64_en-US.msi" -DestinationPath "Peppi_1.0.2_x64_en-US.msi.zip"`

**Sign the zip**

```powershell
bunx tauri signer sign --private-key (Get-Content "C:\path\to\peppi.key" -Raw) --password "your-key-password" Peppi_1.0.2_x64_en-US.msi.zip
```

This creates `Peppi_1.0.2_x64_en-US.msi.zip.sig`. Open that file and copy the whole line (the signature).

---

## 4. Create latest.json

Create a file named `latest.json` in the project root with this content (fill in the placeholders):

- `VERSION` → e.g. `1.0.2`
- `SIGNATURE` → the full line you copied from the `.sig` file
- `PUB_DATE` → current UTC time in the form `2025-02-14T12:00:00Z`
- `NOTES` → short release notes (escape any `"` as `\"`)

```json
{
  "version": "VERSION",
  "notes": "NOTES",
  "pub_date": "PUB_DATE",
  "platforms": {
    "windows-x86_64": {
      "signature": "SIGNATURE",
      "url": "https://pub-a7a1511ed0f84ebbb1afa93d4fe41cb6.r2.dev/msi/Peppi_VERSION_x64_en-US.msi.zip"
    }
  }
}
```

Example for 1.0.2:

```json
{
  "version": "1.0.2",
  "notes": "Bug fixes and improvements",
  "pub_date": "2025-02-14T20:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVk...paste-full-sig-here...",
      "url": "https://pub-a7a1511ed0f84ebbb1afa93d4fe41cb6.r2.dev/msi/Peppi_1.0.2_x64_en-US.msi.zip"
    }
  }
}
```

---

## 5. Upload to R2

Upload these two things to your R2 bucket (same bucket that serves the updater):

1. **File:** `Peppi_1.0.2_x64_en-US.msi.zip`  
   **Path in bucket:** `msi/Peppi_1.0.2_x64_en-US.msi.zip`

2. **File:** `latest.json`  
   **Path in bucket:** `latest.json` (at the root, so the URL is the one in your app’s updater config)

Use the Cloudflare dashboard (R2 → bucket → Upload) or the S3-compatible API / CLI if you use that.

---

## 6. Commit and tag

```powershell
git add src-tauri/tauri.conf.json package.json latest.json
git commit -m "Release v1.0.2"
git tag v1.0.2
git push
git push --tags
```

---

## 7. (Optional) GitHub Release

If you want a GitHub Release page with installers:

1. On GitHub: Repo → Releases → Draft a new release.
2. Choose tag `v1.0.2`.
3. Title: `Release v1.0.2`, add notes.
4. Upload the MSI (and any NSIS `.exe` from `src-tauri\target\release\bundle\nsis\` if you want).
5. Publish.

---

## Checklist

- [ ] Version set in `tauri.conf.json` and `package.json`
- [ ] Built with `--features real-recording` (script does this; if manual: `bun run tauri build -- --features real-recording`) and signing env vars set
- [ ] MSI zipped and signed; signature copied from `.sig` file
- [ ] `latest.json` created with correct version, signature, url, pub_date
- [ ] Zip uploaded to R2 at `msi/Peppi_X.Y.Z_x64_en-US.msi.zip`
- [ ] `latest.json` uploaded to R2 at `latest.json`
- [ ] Git commit + tag `vX.Y.Z` pushed
- [ ] (Optional) GitHub Release created and installers attached
