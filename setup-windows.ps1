param(
    [switch]$Build,
    [switch]$Run,
    [switch]$Test,
    [switch]$Clean,
    [switch]$Help
)

# =============================================================================
# PowerShell Setup & Build Script for Rust Simulation
# =============================================================================
# This script automates the process of building, packaging, and running the
# Rust Simulation on Windows. It includes dependency checking, Rust
# installation, and release packaging.
# =============================================================================

# --- Script Configuration ---
$ErrorActionPreference = "Stop"
# Speed up builds by using the LLD linker.
# This requires the `lld` component to be installed via `rustup component add lld`.
$UseLldLinker = $true
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

function Use-LldLinker {
    if (-not $UseLldLinker) {
        return $false
    }

    Write-Log "Checking for LLD linker component..."
    # Check if rustup is even installed
    if (-not (Get-Command "rustup" -ErrorAction SilentlyContinue)) {
        Write-Log "rustup command not found. Cannot manage components. Assuming LLD is unavailable." -Level "WARN"
        return $false
    }

    # Check for the lld component
    $components = rustup component list
    if ($components -match "lld-linker.*\(installed\)" -or $components -match "rust-lld.*\(installed\)") {
        Write-Log "LLD component is already installed." -Level "SUCCESS"
        return $true
    }

    Write-Log "LLD component not found. Attempting to install..." -Level "WARN"
    try {
        rustup component add lld
        Write-Log "LLD component installed successfully." -Level "SUCCESS"
        return $true
    }
    catch {
        Write-Log "Failed to install the LLD component. Falling back to the default linker." -Level "ERROR"
        Write-Log "Please try installing it manually: `rustup component add lld`" -Level "WARN"
        return $false
    }
}

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

function Update-App {
    Write-Log "Checking for updates..."
    if (-not (Get-Command "git" -ErrorAction SilentlyContinue)) {
        throw "Git command not found. Please install Git and ensure it's in your PATH to use the update feature."
    }
    $gitPath = Join-Path $ScriptDir ".git"
    if (-not (Test-Path $gitPath)) {
        Write-Log "This does not appear to be a Git repository." -Level "WARN"
        Write-Log "Cannot update automatically. Please download the latest version manually." -Level "WARN"
        Read-Host "Press Enter to continue with a rebuild of the current version, or press Ctrl+C to abort."
        return
    }

    Set-Location $ScriptDir

    # Check for local changes before pulling
    $status = git status --porcelain
    if ($status) {
        Write-Log "You have uncommitted local changes." -Level "WARN"
        Write-Log "Pulling updates may result in conflicts." -Level "WARN"
        $choice = Read-Host "Do you want to proceed with the update anyway? (y/N)"
        if ($choice -ne "y") {
            throw "Update cancelled by user. Please commit or stash your changes first."
        }
    }

    Write-Log "Attempting to pull the latest changes..."
    try {
        git pull
        Write-Log "Successfully pulled latest changes. The application will now be rebuilt." -Level "SUCCESS"
    }
    catch {
        throw "git pull failed. Please resolve any conflicts or issues and run the script again."
    }
    finally {
        Set-Location $ScriptDir
    }
}

function Invoke-Tests {
    Write-Log "Running test suite..."
    if (-not (Check-Rust)) {
        Install-Rust
    }

    Set-Location $ProjectDir
    try {
        cargo test
        Write-Log "All tests passed successfully." -Level "SUCCESS"
    }
    catch {
        throw "Tests failed."
    }
    finally {
        Set-Location $ScriptDir
    }
}

function Show-MainMenu {
    $exePath = Join-Path $DistPath "$PackageName.exe"
    if (-not (Test-Path $exePath)) {
        return "rebuild" # No existing build, must rebuild
    }

    Write-Log "An existing build was found."
    Write-Host "What would you like to do?"
    Write-Host "  1. Launch existing version"
    Write-Host "  2. Rebuild the application"
    Write-Host "  3. Check for Updates and Rebuild"
    Write-Host "  4. Run Tests"
    $choice = Read-Host ">"

    switch ($choice) {
        "1" { return "launch" }
        "2" { return "rebuild" }
        "3" {
            Update-App
            return "rebuild" # After updating, we must rebuild
        }
        "4" { return "test" }
        default {
            Write-Log "Invalid option. Aborting." -Level "WARN"
            return "exit"
        }
    }
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

    # Prepare environment for build
    $originalRustFlags = $env:RUSTFLAGS
    if (Use-LldLinker) {
        Write-Log "Attempting to use LLD linker to speed up the build."
        $env:RUSTFLAGS = "$originalRustFlags -C linker=rust-lld.exe"
    }

    try {
        cargo build --release
        Write-Log "Project built successfully." -Level "SUCCESS"
    }
    catch {
        Write-Log "Build failed. Please check the error messages." -Level "ERROR"
        Write-Log "Note: If the error mentions 'linker' or 'LNK' errors, you may need to install the 'C++ build tools' from the Visual Studio Installer." -Level "WARN"
        if ($UseLldLinker) {
            Write-Log "LLD linker might have failed. Try running with `$UseLldLinker = `$false` in the script." -Level "WARN"
        }
        throw "Build failed."
    }
    finally {
        # Restore original environment
        $env:RUSTFLAGS = $originalRustFlags
        Set-Location $ScriptDir
    }
}

function Package-App {
    param([string]$Version)

    Write-Log "Packaging the application..."

    # --- Resilience Checks ---
    if (Test-Path $DistDir -PathType Leaf) {
        throw "A file named 'dist' exists in the project root. Please remove it before packaging."
    }
    if (-not (Test-Path (Join-Path $ProjectDir "data") -PathType Container)) {
        throw "Source data directory not found at '$ProjectDir\data'. Cannot package release."
    }

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

function Get-OSVersion {
    try {
        # Modern approach for PowerShell 5.1+
        $osInfo = Get-ComputerInfo | Select-Object "WindowsProductName", "WindowsVersion", "OsBuildNumber"
        Write-Log "OS Detected: $($osInfo.WindowsProductName), Version $($osInfo.WindowsVersion), Build $($osInfo.OsBuildNumber)"
    }
    catch {
        # Fallback for older PowerShell versions
        try {
            $osInfo = Get-CimInstance Win32_OperatingSystem | Select-Object "Caption", "Version"
            Write-Log "OS Detected: $($osInfo.Caption), Version $($osInfo.Version)"
        }
        catch {
            Write-Log "Could not determine detailed Windows version." -Level "WARN"
        }
    }
}

function Show-Help {
    Write-Host "Usage: ./setup-windows.ps1 [options]"
    Write-Host
    Write-Host "This script builds, runs, and tests the Rust Simulation project."
    Write-Host
    Write-Host "Options:"
    Write-Host "  -Build          Build the application."
    Write-Host "  -Run            Run the application after building. Implies -Build."
    Write-Host "  -Test           Run the project's test suite."
    Write-Host "  -Clean          Perform a clean build before any action."
    Write-Host "  -Help           Display this help message and exit."
    Write-Host
    Write-Host "If no options are provided, the script will start in interactive mode."
    exit 0
}

# =============================================================================
# Main Script
# =============================================================================
if ($Help) { Show-Help }

# Determine run mode
$Interactive = ($PSBoundParameters.Keys.Count -eq 0)

if ($Run) { $Build = $true } # -Run implies -Build

Clear-Host
Write-Log "Welcome to the Rust Simulation Setup Script"
Write-Log "=========================================="

Get-OSVersion

try {
    if ($Interactive) {
        $action = Show-MainMenu
        switch ($action) {
            "launch" { Launch-App; exit }
            "exit"   { exit }
            "test"   { $Test = $true }
            # "rebuild" will just fall through
        }
    }

    # --- Non-Interactive and Post-Menu Logic ---

    if ($Test) {
        Invoke-Tests
    }

    if ($Build) {
        $projectVersion = Get-ProjectVersion
        $doClean = $false
        if ($Interactive) {
            $doClean = Ask-CleanBuild
        } elseif ($Clean) {
            $doClean = $true
        }
        if (-not (Check-Rust)) {
            Install-Rust
        }
        Build-Project -Clean $doClean
        Package-App -Version $projectVersion

        if ($Run) {
            Launch-App
        } else {
            Write-Log "Build complete. Use -Run to launch the application."
        }
    } elseif ($Interactive -and $action -eq "rebuild") {
        # This handles the case where the user chose "rebuild" from the menu
        # without any non-interactive flags being set.
        $projectVersion = Get-ProjectVersion
        $doClean = Ask-CleanBuild
        if (-not (Check-Rust)) {
            Install-Rust
        }
        Build-Project -Clean $doClean
        Package-App -Version $projectVersion
        Launch-App
    }

    Write-Log "Script finished successfully." -Level "SUCCESS"
}
catch {
    Write-Log "The script encountered a critical error: $($_.Exception.Message)" -Level "ERROR"
}
finally {
    Read-Host "Press Enter to exit."
}
