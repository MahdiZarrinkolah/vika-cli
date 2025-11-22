# PowerShell installation script for vika-cli

$ErrorActionPreference = "Stop"

$REPO = "MahdiZarrinkolah/vika-cli"
$BINARY_NAME = "vika-cli"
$ASSET_NAME = "vika-cli-windows-x86_64.exe"

# Colors for output
function Write-Success {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

# Get latest release version
function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        return $response.tag_name
    }
    catch {
        Write-Error "Error: Could not determine latest version"
        exit 1
    }
}

# Download binary
function Download-Binary {
    param(
        [string]$Version,
        [string]$DownloadPath
    )
    
    $DownloadUrl = "https://github.com/$REPO/releases/download/$Version/$ASSET_NAME"
    $ChecksumUrl = "https://github.com/$REPO/releases/download/$Version/$ASSET_NAME.sha256"
    
    Write-Success "Downloading $BINARY_NAME $Version..."
    
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $DownloadPath -UseBasicParsing
        Invoke-WebRequest -Uri $ChecksumUrl -OutFile "$DownloadPath.sha256" -UseBasicParsing
    }
    catch {
        Write-Error "Error: Failed to download binary"
        exit 1
    }
    
    # Verify checksum
    Write-Success "Verifying checksum..."
    $ExpectedHash = (Get-Content "$DownloadPath.sha256" -Raw).Split()[0]
    $ActualHash = (Get-FileHash -Path $DownloadPath -Algorithm SHA256).Hash.ToLower()
    
    if ($ExpectedHash -eq $ActualHash) {
        Write-Success "Checksum verified"
    }
    else {
        Write-Error "Error: Checksum verification failed"
        exit 1
    }
}

# Install binary
function Install-Binary {
    param([string]$BinaryPath)
    
    $InstallDir = "$env:USERPROFILE\AppData\Local\Programs\vika-cli"
    $InstallPath = "$InstallDir\$BINARY_NAME.exe"
    
    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # Copy binary
    Copy-Item -Path $BinaryPath -Destination $InstallPath -Force
    Write-Success "Installed to $InstallPath"
    
    # Add to PATH if not already present
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        $NewPath = "$UserPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        Write-Success "Added $InstallDir to PATH"
        Write-Warning "Please restart your terminal for PATH changes to take effect"
    }
    else {
        Write-Success "$InstallDir is already in PATH"
    }
    
    Write-Success "✓ $BINARY_NAME installed successfully!"
    Write-Success "Run '$BINARY_NAME --help' to get started"
}

# Main
function Main {
    Write-Success "Installing $BINARY_NAME..."
    
    $Version = Get-LatestVersion
    $TempPath = "$env:TEMP\$ASSET_NAME"
    
    Download-Binary -Version $Version -DownloadPath $TempPath
    Install-Binary -BinaryPath $TempPath
    
    # Cleanup
    Remove-Item $TempPath -ErrorAction SilentlyContinue
    Remove-Item "$TempPath.sha256" -ErrorAction SilentlyContinue
}

Main

