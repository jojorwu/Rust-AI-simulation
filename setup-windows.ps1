# =============================================================================
# PowerShell Setup & Build Script for Rust Simulation
# =============================================================================
# This script automates the process of building, packaging, and running the
# Rust Simulation on Windows. It includes dependency checking, Rust
# installation, and release packaging.
# =============================================================================

# --- Script Configuration ---
$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Join-Path $ScriptDir "rust_simulation"
$DistDir = Join-Path $ScriptDir "dist"
$DistPath = Join-Path $DistDir "windows"
$PackageName = "rust_simulation"

# --- Color-Coded Logging Functions ---
function Write-Log {
    param (
        [string]$Message,
        [string]$Level = "INFO"
    )
    $color = @{
        "INFO"  = "Cyan";
        "WARN"  = "Yellow";
        "ERROR" = "Red";
        "SUCCESS" = "Green"
    }
    Write-Host "[$Level] -" $Message -ForegroundColor $color[$Level]
}

# =============================================================================
# Helper Functions
# =============================================================================

function Get-ProjectVersion {
    Write-Log "Getting project version..."
    $tomlPath = Join-Path $ProjectDir "Cargo.toml"
    if (-not (Test-Path $tomlPath)) {
        throw "Cargo.toml not found at '$tomlPath'"
    }
    try {
        $inPackageSection = $false
        foreach ($line in (Get-Content -Path $tomlPath)) {
            $trimmedLine = $line.Trim()
            if ($trimmedLine -eq '[package]') { $inPackageSection = $true; continue }
            if ($trimmedLine.StartsWith('[') -and $trimmedLine.EndsWith(']')) { $inPackageSection = $false }
            if ($inPackageSection -and $trimmedLine -match '^version\s*=\s*') {
                $version = ($trimmedLine.Split('=')[1]).Trim().Trim('"')
                Write-Log "Found version: $version" -Level "SUCCESS"
                return $version
            }
        }
    }
    catch {
        Write-Log "Could not reliably determine project version. Defaulting to 'unknown'." -Level "WARN"
        return "unknown"
    }
}

function Check-ExistingBuild {
    $exePath = Join-Path $DistPath "$PackageName.exe"
    if (Test-Path $exePath) {
        Write-Log "An existing build was found."
        $choice = Read-Host "What would you like to do? (1) Launch existing version (2) Rebuild the application"
        if ($choice -eq "1") {
            return $false # Do not rebuild
        }
    }
    return $true # Rebuild
}

function Ask-CleanBuild {
    $choice = Read-Host "Perform a clean build? (This is slower but can fix some issues) (y/N)"
    return ($choice -eq "y")
}

function Check-Rust {
    $cargoExists = (Get-Command "cargo" -ErrorAction SilentlyContinue)
    if ($cargoExists) {
        Write-Log "Rust (cargo) is installed."
        return $true
    } else {
        Write-Log "Rust (cargo) not found." -Level "WARN"
        return $false
    }
}

function Install-Rust {
    $choice = Read-Host "Would you like to install Rust now? (y/N)"
    if ($choice -ne "y") {
        throw "Rust installation skipped. Cannot proceed."
    }

    $installerUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $checksumUrl = "$installerUrl.sha256"
    $installerPath = Join-Path $ScriptDir "rustup-init.exe"

    try {
        Write-Log "Downloading Rust installer from '$installerUrl'..."
        Invoke-WebRequest -Uri $installerUrl -OutFile $installerPath

        Write-Log "Downloading checksum..."
        $officialChecksumLine = Invoke-WebRequest -Uri $checksumUrl
        $officialChecksum = ($officialChecksumLine.Content -split ' ')[0].ToUpper()

        Write-Log "Calculating local file hash..."
        $localChecksum = (Get-FileHash -Path $installerPath -Algorithm SHA256).Hash.ToUpper()

        Write-Log "Official Checksum: $officialChecksum"
        Write-Log "Local Checksum:    $localChecksum"

        if ($localChecksum -ne $officialChecksum) {
            throw "Checksum mismatch! The downloaded file may be corrupt or tampered with."
        }
        Write-Log "Checksums match. File is valid." -Level "SUCCESS"

        Write-Log "Starting the Rust installer. Please follow the on-screen instructions."
        Start-Process -FilePath $installerPath -ArgumentList "--default-toolchain stable -y" -Wait

        Write-Log "IMPORTANT: You must restart your terminal for the new PATH to take effect. Please close this window and run the script again." -Level "WARN"
        Read-Host "Press Enter to exit."
        exit
    }
    catch {
        throw "An error occurred during Rust installation: $($_.Exception.Message)"
    }
    finally {
        if (Test-Path $installerPath) {
            Remove-Item $installerPath
        }
    }
}

function Build-Project {
    param([bool]$Clean)

    Write-Log "Building project... This may take a few minutes."
    Set-Location $ProjectDir

    if ($Clean) {
        Write-Log "Cleaning previous build artifacts..."
        cargo clean | Out-Null
    }

    try {
        cargo build --release
        Write-Log "Project built successfully." -Level "SUCCESS"
    }
    catch {
        Write-Log "Build failed. Please check the error messages." -Level "ERROR"
        Write-Log "Note: If the error mentions 'linker' or 'LNK' errors, you may need to install the 'C++ build tools' from the Visual Studio Installer." -Level "WARN"
        throw "Build failed."
    }
    finally {
        Set-Location $ScriptDir
    }
}

function Package-App {
    param([string]$Version)

    Write-Log "Packaging the application..."
    New-Item -ItemType Directory -Path $DistPath -Force | Out-Null

    $sourceExe = Join-Path $ProjectDir "target/release/$PackageName.exe"
    $destExe = Join-Path $DistPath "$PackageName.exe"
    $sourceData = Join-Path $ProjectDir "data"
    $destData = Join-Path $DistPath "data"

    Write-Log "  - Copying executable..."
    Copy-Item -Path $sourceExe -Destination $destExe -Force
    Write-Log "  - Copying data files..."
    Copy-Item -Path $sourceData -Destination $destData -Recurse -Force

    # Create README
    $readmeContent = @"
==================================
Rust Simulation - Instructions
==================================

Thank you for downloading Rust Simulation!

To run the game, simply run the `${PackageName}.exe` file.

If the game does not start and you see an error about a missing DLL (like VCRUNTIME140.dll),
please run the included `vc_redist.x64.exe` installer first. This is the official
Microsoft C++ library installer and is required by the game engine.
"@
    $readmeContent | Out-File -FilePath (Join-Path $DistPath "README.txt") -Encoding "utf8"

    # Bundle VC++ Redistributable
    Write-Log "Bundling Microsoft VC++ Redistributable..."
    $redistUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    $redistPath = Join-Path $DistPath "vc_redist.x64.exe"
    try {
        Invoke-WebRequest -Uri $redistUrl -OutFile $redistPath
    } catch {
        Write-Log "Failed to download VC++ Redistributable. The application may not run on all PCs." -Level "WARN"
    }

    # Create ZIP archive
    Write-Log "Creating ZIP archive..."
    $zipPath = Join-Path $DistDir "${PackageName}_v${Version}_windows.zip"
    Compress-Archive -Path "$DistPath\*" -DestinationPath $zipPath -Force

    Write-Log "Application packaged successfully!" -Level "SUCCESS"
    Write-Log "The distributable is in '$DistPath' and '$zipPath'."
}

function Launch-App {
    Write-Log "Launching the application..."
    $exePath = Join-Path $DistPath "$PackageName.exe"
    Start-Process -FilePath $exePath -WorkingDirectory $DistPath
}

# =============================================================================
# Main Script
# =============================================================================
Clear-Host
Write-Log "Welcome to the Rust Simulation Setup Script"
Write-Log "=========================================="

try {
    # Check for existing build and ask user what to do
    if (-not (Check-ExistingBuild)) {
        Launch-App
        exit
    }

    # Get project version
    $projectVersion = Get-ProjectVersion

    # Ask about clean build
    $doClean = Ask-CleanBuild

    # Check for and install Rust
    if (-not (Check-Rust)) {
        Install-Rust
        # The script will exit inside Install-Rust if it runs the installer
    }

    # Build, package, and launch
    Build-Project -Clean $doClean
    Package-App -Version $projectVersion
    Launch-App

    Write-Log "Script finished successfully." -Level "SUCCESS"
}
catch {
    Write-Log "The script encountered a critical error: $($_.Exception.Message)" -Level "ERROR"
}
finally {
    Read-Host "Press Enter to exit."
}
