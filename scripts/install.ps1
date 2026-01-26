#Requires -Version 5.1
<#
.SYNOPSIS
    qmpo installer for Windows

.DESCRIPTION
    Downloads and installs qmpo from GitHub Releases.
    Registers the directory:// URI scheme.

.PARAMETER Silent
    Run in silent mode without prompts

.PARAMETER InstallDir
    Installation directory (default: $env:LOCALAPPDATA\qmpo)

.PARAMETER Version
    Specific version to install (default: latest)

.EXAMPLE
    # Interactive installation
    .\install.ps1

    # Silent installation
    .\install.ps1 -Silent

    # Install specific version
    .\install.ps1 -Version v0.2.0
#>

param(
    [switch]$Silent,
    [string]$InstallDir = "$env:LOCALAPPDATA\qmpo",
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$RepoOwner = "tagawa0525"
$RepoName = "qmpo"
$ArtifactName = "qmpo-windows-x64.zip"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $output = "[$timestamp] [$Level] $Message"
    if ($Level -eq "ERROR") {
        Write-Error $output
    } elseif (-not $Silent) {
        Write-Host $output
    }
}

function Get-LatestRelease {
    $url = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
    try {
        $response = Invoke-RestMethod -Uri $url -Method Get
        return $response.tag_name
    } catch {
        Write-Log "Failed to get latest release: $_" "ERROR"
        exit 1
    }
}

function Get-ReleaseDownloadUrl {
    param([string]$Tag)
    return "https://github.com/$RepoOwner/$RepoName/releases/download/$Tag/$ArtifactName"
}

function Install-Qmpo {
    Write-Log "Starting qmpo installation..."

    # Determine version
    if ($Version -eq "latest") {
        Write-Log "Fetching latest version..."
        $Version = Get-LatestRelease
    }
    Write-Log "Installing version: $Version"

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        Write-Log "Created directory: $InstallDir"
    }

    # Download
    $downloadUrl = Get-ReleaseDownloadUrl -Tag $Version
    $zipPath = Join-Path $env:TEMP "qmpo-download.zip"

    Write-Log "Downloading from: $downloadUrl"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
    } catch {
        Write-Log "Failed to download: $_" "ERROR"
        exit 1
    }

    # Extract
    Write-Log "Extracting to: $InstallDir"
    Expand-Archive -Path $zipPath -DestinationPath $InstallDir -Force

    # Clean up
    Remove-Item -Path $zipPath -Force

    # Register URI scheme
    Write-Log "Registering directory:// URI scheme..."
    Register-UriScheme

    # Add to PATH (user level)
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")
        Write-Log "Added $InstallDir to user PATH"
    }

    Write-Log "Installation completed successfully!"
    Write-Log "You may need to restart your terminal for PATH changes to take effect."
}

function Register-UriScheme {
    $qmpoPath = Join-Path $InstallDir "qmpo.exe"

    if (-not (Test-Path $qmpoPath)) {
        Write-Log "qmpo.exe not found at $qmpoPath" "ERROR"
        exit 1
    }

    # Register directory:// URI scheme in HKCU (no admin required)
    $regPath = "HKCU:\Software\Classes\directory"

    New-Item -Path $regPath -Force | Out-Null
    Set-ItemProperty -Path $regPath -Name "(Default)" -Value "URL:Directory Protocol"
    Set-ItemProperty -Path $regPath -Name "URL Protocol" -Value ""

    New-Item -Path "$regPath\shell\open\command" -Force | Out-Null
    Set-ItemProperty -Path "$regPath\shell\open\command" -Name "(Default)" -Value "`"$qmpoPath`" `"%1`""

    Write-Log "Registered directory:// URI scheme"
}

# Main
try {
    Install-Qmpo
} catch {
    Write-Log "Installation failed: $_" "ERROR"
    exit 1
}
