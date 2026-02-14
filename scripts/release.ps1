# Peppi Release Build Script
# Usage: .\release.ps1 -Version "1.0.2"
# Or:    .\release.ps1 (will prompt for version)

param(
    [string]$Version,
    [string]$Notes = "",
    [string]$KeyPath = "C:\Users\cafebabe\peppi-keys\peppi.key",
    [switch]$SkipBuild,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Step { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Write-Info { param($msg) Write-Host "  $msg" -ForegroundColor White }
function Write-Success { param($msg) Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host "  [!] $msg" -ForegroundColor Yellow }

# Project paths
$ProjectRoot = Split-Path $PSScriptRoot -Parent
$TauriConf = Join-Path $ProjectRoot "src-tauri\tauri.conf.json"
$PackageJson = Join-Path $ProjectRoot "package.json"
$LatestJson = Join-Path $ProjectRoot "latest.json"

Write-Host ""
Write-Host "========================================" -ForegroundColor Magenta
Write-Host "       PEPPI RELEASE BUILDER           " -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta

# Get current version
$currentTauriConf = Get-Content $TauriConf | ConvertFrom-Json
$currentVersion = $currentTauriConf.version
Write-Host ""
Write-Info "Current version: $currentVersion"

# Prompt for version if not provided
if (-not $Version) {
    $Version = Read-Host 'Enter new version (e.g. 1.0.2)'
    if (-not $Version) {
        Write-Error "Version is required"
        exit 1
    }
}

# Validate version format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error 'Invalid version format. Expected: X.Y.Z (e.g. 1.0.2)'
    exit 1
}

Write-Info "New version: $Version"

if ($DryRun) {
    Write-Warning "DRY RUN - No changes will be made"
}

# ============================================================================
# STEP 1: Update version in all files
# ============================================================================
Write-Step "Updating version in project files"

# Update tauri.conf.json
Write-Info "Updating src-tauri/tauri.conf.json..."
if (-not $DryRun) {
    $tauriContent = Get-Content $TauriConf -Raw
    $tauriContent = $tauriContent -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
    Set-Content $TauriConf $tauriContent -NoNewline
}
Write-Success "tauri.conf.json updated"

# Update package.json
Write-Info "Updating package.json..."
if (-not $DryRun) {
    $pkgContent = Get-Content $PackageJson -Raw
    $pkgContent = $pkgContent -replace '"version":\s*"[^"]*"', "`"version`": `"$Version`""
    Set-Content $PackageJson $pkgContent -NoNewline
}
Write-Success "package.json updated"

# ============================================================================
# STEP 2: Build the application
# ============================================================================
if (-not $SkipBuild) {
    Write-Step "Building Tauri application"
    
    # Check for signing key
    if (-not (Test-Path $KeyPath)) {
        Write-Error "Signing key not found at: $KeyPath"
        exit 1
    }
    
    # Read the private key
    $privateKey = Get-Content $KeyPath -Raw
    
    # Set environment variables for signing
    $env:TAURI_SIGNING_PRIVATE_KEY = $privateKey
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "falcoissocringe123!"
    
    Write-Info "Starting build with signing enabled..."
    Write-Info "(This may take several minutes)"
    
    if (-not $DryRun) {
        Push-Location $ProjectRoot
        try {
            # Run the build (real-recording = actual screen capture)
            bun run tauri build -- --features real-recording
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Build failed!"
                exit 1
            }
        } finally {
            Pop-Location
            # Clear sensitive env vars
            Remove-Item Env:\TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
            Remove-Item Env:\TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
        }
    }
    Write-Success "Build completed"
} else {
    Write-Warning 'Skipping build (-SkipBuild flag set)'
}

# ============================================================================
# STEP 3: Find and process the MSI
# ============================================================================
Write-Step "Processing build artifacts"

$msiPattern = "Peppi_${Version}_x64_en-US.msi"
$bundleDir = Join-Path $ProjectRoot "src-tauri\target\release\bundle\msi"
$msiPath = Join-Path $bundleDir $msiPattern

if (-not $DryRun) {
    if (-not (Test-Path $msiPath)) {
        Write-Error "MSI not found at: $msiPath"
        Write-Info "Looking for MSI files in bundle directory..."
        Get-ChildItem $bundleDir -Filter "*.msi" | ForEach-Object { Write-Info "  Found: $($_.Name)" }
        exit 1
    }
}

$zipPath = "$msiPath.zip"
$sigPath = "$zipPath.sig"

Write-Info "MSI: $msiPattern"

# Create ZIP
Write-Info "Creating ZIP archive..."
if (-not $DryRun) {
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    Compress-Archive -Path $msiPath -DestinationPath $zipPath -Force
}
Write-Success "ZIP created"

# ============================================================================
# STEP 4: Sign the update
# ============================================================================
Write-Step "Signing update package"

if (-not $DryRun) {
    $privateKey = Get-Content $KeyPath -Raw
    $signResult = bunx tauri signer sign --private-key $privateKey --password "falcoissocringe123!" $zipPath 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Signing failed: $signResult"
        exit 1
    }
    
    # Get signature
    if (Test-Path $sigPath) {
        $signature = (Get-Content $sigPath -Raw).Trim()
    } else {
        $signatureMatch = $signResult | Select-String -Pattern "^dW50cnVzdGVk.*$" -AllMatches
        if ($signatureMatch) {
            $signature = $signatureMatch.Matches[0].Value
        } else {
            Write-Error "Could not extract signature"
            exit 1
        }
    }
} else {
    $signature = "SIGNATURE_PLACEHOLDER_FOR_DRY_RUN"
}

Write-Success "Signed successfully"

# ============================================================================
# STEP 5: Update latest.json
# ============================================================================
Write-Step "Updating latest.json"

$pubDate = Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ"
$downloadUrl = "https://pub-a7a1511ed0f84ebbb1afa93d4fe41cb6.r2.dev/msi/${msiPattern}.zip"

# Prompt for release notes if not provided
if (-not $Notes) {
    Write-Host ""
    $Notes = Read-Host "Enter release notes (or press Enter to skip)"
    if (-not $Notes) {
        $Notes = "Bug fixes and improvements"
    }
}

$notesEscaped = $Notes -replace '\\', '\\\\' -replace '"', '\"'
$latestContent = '{"version":"' + $Version + '","notes":"' + $notesEscaped + '","pub_date":"' + $pubDate + '","platforms":{"windows-x86_64":{"signature":"' + $signature + '","url":"' + $downloadUrl + '"}}}'

Write-Info "Updating latest.json..."
if (-not $DryRun) {
    Set-Content $LatestJson $latestContent
}
Write-Success "latest.json updated"

# ============================================================================
# SUMMARY
# ============================================================================
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "         RELEASE COMPLETE!              " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Version: $Version" -ForegroundColor White
Write-Host "Notes: $Notes" -ForegroundColor White
Write-Host ""

Write-Host "Files to upload to R2:" -ForegroundColor Yellow
Write-Host "  1. $zipPath" -ForegroundColor White
Write-Host ""

Write-Host "Files to commit to git:" -ForegroundColor Yellow
Write-Host "  1. latest.json" -ForegroundColor White
Write-Host "  2. src-tauri/tauri.conf.json" -ForegroundColor White
Write-Host "  3. package.json" -ForegroundColor White
Write-Host ""

Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Upload ZIP to R2: msi/${msiPattern}.zip" -ForegroundColor White
Write-Host "  2. Upload latest.json to R2 (or commit to repo)" -ForegroundColor White
Write-Host ('  3. git add -A; git commit -m "Release v' + $Version + '"') -ForegroundColor White
Write-Host "  4. git tag v$Version; git push --tags" -ForegroundColor White
Write-Host ""

# Copy signature to clipboard
if (-not $DryRun) {
    $signature | Set-Clipboard
    Write-Success "Signature copied to clipboard"
}
