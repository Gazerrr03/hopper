# hopper Windows installer
# Run in PowerShell: irm https://raw.githubusercontent.com/qizhidong/hopper/main/install.ps1 | iex

$REPO = "qizhidong/hopper"
$TAG = (Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest").tag_name -replace 'v', ''

$ARCH = if ($env:PROCESSOR_ARCHITECTURE -eq "AMD64") { "x86_64" } else { "aarch64" }
$OS = "pc-windows"
$BINARY = "hopper-${OS}-${ARCH}.tar.gz"
$URL = "https://github.com/${REPO}/releases/download/v${TAG}/${BINARY}"

$OUT = "$env:LOCALAPPDATA\hopper"
$EXEC = "$OUT\hopper.exe"

Write-Host "Installing hopper v${TAG} for ${OS}-${ARCH}..."

# Download
New-Item -ItemType Directory -Force -Path $OUT | Out-Null
Invoke-WebRequest -Uri $URL -OutFile "$OUT\$BINARY" -UseBasicParsing

# Extract (tar is not native on Windows, use tar from git or 7zip if available)
$tar = Get-Command tar -ErrorAction SilentlyContinue
if ($tar) {
    tar -xzf "$OUT\$BINARY" -C $OUT
} else {
    Write-Host "tar not found. Please install tar or extract manually:"
    Write-Host $URL
    exit 1
}

Write-Host "Installed to $EXEC"
Write-Host "Add $OUT to your PATH"
