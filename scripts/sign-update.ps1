# Peppi Update Signing Script
# Usage: .\sign-update.ps1 -MsiPath "path\to\Peppi_x.x.x_x64_en-US.msi"

param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath,
    
    [string]$KeyPath = "C:\Users\cafebabe\peppi-keys\peppi.key",
    [string]$KeyPassword = "falcoissocringe123!"
)

# Validate MSI exists
if (-not (Test-Path $MsiPath)) {
    Write-Error "MSI file not found: $MsiPath"
    exit 1
}

# Get the directory and filename
$msiDir = Split-Path $MsiPath -Parent
$msiName = Split-Path $MsiPath -Leaf
$zipPath = "$MsiPath.zip"
$sigPath = "$zipPath.sig"

Write-Host "Creating update package for: $msiName" -ForegroundColor Cyan

# Step 1: Create ZIP archive
Write-Host "Step 1: Creating ZIP archive..." -ForegroundColor Yellow
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}
Compress-Archive -Path $MsiPath -DestinationPath $zipPath -Force

if (-not (Test-Path $zipPath)) {
    Write-Error "Failed to create ZIP archive"
    exit 1
}

$zipSize = (Get-Item $zipPath).Length
Write-Host "  Created: $zipPath ($zipSize bytes)" -ForegroundColor Green

# Step 2: Sign the ZIP
Write-Host "Step 2: Signing ZIP archive..." -ForegroundColor Yellow

# Read the private key
if (-not (Test-Path $KeyPath)) {
    Write-Error "Private key not found: $KeyPath"
    exit 1
}

$keyContent = Get-Content $KeyPath -Raw

# Sign using tauri signer
$signResult = bunx tauri signer sign --private-key $keyContent --password $KeyPassword $zipPath 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to sign ZIP: $signResult"
    exit 1
}

# Extract the signature from the output
$signatureMatch = $signResult | Select-String -Pattern "^dW50cnVzdGVk.*$" -AllMatches
if ($signatureMatch) {
    $signature = $signatureMatch.Matches[0].Value
} else {
    # Read from the .sig file
    if (Test-Path $sigPath) {
        $signature = Get-Content $sigPath -Raw
        $signature = $signature.Trim()
    } else {
        Write-Error "Could not find signature"
        exit 1
    }
}

Write-Host "  Signed successfully!" -ForegroundColor Green

# Step 3: Output summary
Write-Host ""
Write-Host "=== UPDATE PACKAGE READY ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Files to upload to R2:" -ForegroundColor Yellow
Write-Host "  1. $zipPath"
Write-Host "  2. latest.json (update with signature below)"
Write-Host ""
Write-Host "Signature for latest.json:" -ForegroundColor Yellow
Write-Host ""
Write-Host $signature -ForegroundColor White
Write-Host ""

# Copy signature to clipboard
$signature | Set-Clipboard
Write-Host "(Signature copied to clipboard)" -ForegroundColor Green
Write-Host ""

# Show the expected latest.json format
$version = if ($msiName -match "Peppi_(\d+\.\d+\.\d+)") { $Matches[1] } else { "x.x.x" }
Write-Host "Example latest.json:" -ForegroundColor Yellow
Write-Host @"
{
    "version": "$version",
    "notes": "Update notes here",
    "pub_date": "$(Get-Date -Format 'yyyy-MM-ddTHH:mm:ssZ')",
    "platforms": {
      "windows-x86_64": {
        "signature": "$signature",
        "url": "https://pub-a7a1511ed0f84ebbb1afa93d4fe41cb6.r2.dev/msi/$($msiName).zip"
      }
    }
}
"@
