#!/bin/bash

# =============================================================================
# Unified Build & Run Script for Rust Simulation
# =============================================================================
# This script is the single entry point for building and running the project.
# It detects the operating system and calls the appropriate setup logic.
#
# Supported OS:
#   - Linux (Debian/Ubuntu, Fedora, Arch)
#   - Windows (via Git Bash or by running the .bat wrapper)
#   - macOS
# =============================================================================

# --- Stop on any error ---
set -e

# --- Script directory ---
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# --- Color Codes for logging ---
C_RESET='\033[0m'
C_RED='\033[0;31m'
C_GREEN='\033[0;32m'
C_YELLOW='\033[0;33m'
C_BLUE='\033[0;34m'

# =============================================================================
# Logging Helper Functions
# =============================================================================
info() {
    echo -e "${C_BLUE}INFO:${C_RESET} $1"
}
warn() {
    echo -e "${C_YELLOW}WARN:${C_RESET} $1"
}
error() {
    echo -e "${C_RED}ERROR:${C_RESET} $1" >&2
    exit 1
}

# =============================================================================
# OS-Specific Logic Placeholder Functions
# =============================================================================

run_linux() {
    # --- OS Version Detection ---
    local os_pretty_name="Linux (Unknown Version)"
    if [ -f /etc/os-release ]; then
        # Source the file to get variables like PRETTY_NAME
        . /etc/os-release
        os_pretty_name="$PRETTY_NAME"
    fi
    info "Linux operating system detected: $os_pretty_name"

    # --- Configuration ---
    local project_dir="$SCRIPT_DIR/rust_simulation"
    local dist_dir="$SCRIPT_DIR/dist"
    local dist_path="$dist_dir/linux"
    local package_name="rust_simulation"
    local project_version="unknown"
    local do_clean=0

    # --- Dependency Definitions ---
    local deps_debian="build-essential libasound2-dev libudev-dev"
    local deps_fedora="alsa-lib-devel libudev-devel systemd-devel"
    local deps_arch="base-devel alsa-lib"

    # =========================================================================
    # Linux-Specific Helper Functions
    # =========================================================================

    check_linux_dependencies() {
        info "Checking for required command-line tools..."
        local missing_tool=0
        for tool in curl grep sed tar awk sha256sum; do
            if ! command -v "$tool" &> /dev/null; then
                warn "Command '$tool' is not found, but is required."
                missing_tool=1
            fi
        done
        if [ "$missing_tool" -eq 1 ]; then
            error "Please install the missing tools and run the script again."
        fi
    }

    install_rust_linux() {
        if command -v "cargo" &> /dev/null; then
            info "Rust is already installed."
            return
        fi

        warn "Rust (cargo) not found."
        read -p "Would you like to install it now using the official rustup script? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            error "Rust installation skipped. Cannot proceed."
        fi

        info "Downloading rustup-init.sh and its checksum..."
        local installer_path="/tmp/rustup-init.sh"
        local installer_url="https://static.rust-lang.org/rustup/rustup-init.sh"
        local checksum_url="${installer_url}.sha256"

        if ! curl -sSf -o "$installer_path" "$installer_url"; then
            error "Failed to download the Rust installer script."
        fi
        if ! curl -sSf -o "${installer_path}.sha256" "$checksum_url"; then
            error "Failed to download the checksum file."
        fi

        info "Verifying installer checksum..."
        local expected_checksum=$(cat "${installer_path}.sha256" | cut -d' ' -f1)
        local actual_checksum=$(sha256sum "$installer_path" | cut -d' ' -f1)

        if [ "$expected_checksum" != "$actual_checksum" ]; then
            error "Checksum mismatch for rustup-init.sh. Aborting installation."
        fi
        info "Checksum verified successfully."

        info "Running the Rust installer..."
        if ! sh "$installer_path" -y; then
            error "Rust installation failed."
        fi

        # Clean up
        rm "$installer_path" "${installer_path}.sha256"

        # Source Cargo environment to make it available in the current session
        source "$HOME/.cargo/env"
        info "Rust installed successfully."
        warn "Please restart your terminal after this script finishes for the changes to take full effect."
    }

    install_system_deps_linux() {
        info "Checking for system dependencies..."
        if [ ! -f /etc/os-release ]; then
            warn "Could not determine Linux distribution. Cannot automatically check system dependencies."
            return
        fi
        local os_id=$(grep -oP '(?<=^ID=).+' /etc/os-release | tr -d '"')

        local pkgs_needed=""
        local pkgs_to_install=""
        local check_cmd=""
        local install_cmd=""

        case "$os_id" in
            "ubuntu" | "debian" | "pop")
                pkgs_needed=$deps_debian; check_cmd="dpkg -s"; install_cmd="sudo apt-get install -y" ;;
            "fedora")
                pkgs_needed=$deps_fedora; check_cmd="rpm -q"; install_cmd="sudo dnf install -y" ;;
            "arch")
                pkgs_needed=$deps_arch; check_cmd="pacman -Qs"; install_cmd="sudo pacman -S --noconfirm" ;;
            *)
                warn "Unsupported Linux distribution '$os_id'. Cannot check system dependencies."
                return ;;
        esac

        for pkg in $pkgs_needed; do
            if ! $check_cmd "$pkg" &> /dev/null; then
                pkgs_to_install="$pkgs_to_install $pkg"
            fi
        done

        if [ -n "$pkgs_to_install" ]; then
            warn "The following system dependencies are required:$pkgs_to_install"
            read -p "Would you like to install them now? (This will use sudo) (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                if ! $install_cmd $pkgs_to_install; then
                    error "Failed to install system dependencies. Please try to install them manually."
                fi
                info "System dependencies installed successfully."
            else
                warn "Installation of system dependencies skipped. The build may fail."
            fi
        else
            info "All required system dependencies are already installed."
        fi
    }

    get_project_version_linux() {
        info "Getting project version..."
        if [ ! -f "$project_dir/Cargo.toml" ]; then
            error "Cargo.toml not found in $project_dir"
        fi
        local version=$(awk 'BEGIN{in_pkg=0} /\[package\]/{in_pkg=1} /^\[/{if(!/\[package\]/)in_pkg=0} in_pkg&&/version/{match($0,/"([^"]+)"/);print substr($0,RSTART+1,RLENGTH-2);exit}' "$project_dir/Cargo.toml")
        if [ -n "$version" ]; then
            project_version="$version"
        else
            warn "Could not reliably determine project version. Defaulting to 'unknown'."
        fi
    }

    launch_app_linux() {
        info "Launching application..."
        cd "$dist_path"
        "./$package_name"
    }

    update_app_linux() {
        info "Checking for updates..."
        if ! command -v git &> /dev/null; then
            error "Git command not found. Please install Git to use the update feature."
        fi
        if [ ! -d "$SCRIPT_DIR/.git" ]; then
            warn "This does not appear to be a Git repository."
            warn "Cannot update automatically. Please download the latest version manually."
            read -p "Press Enter to continue with a rebuild of the current version, or Ctrl+C to abort."
            return
        fi

        info "Attempting to pull the latest changes..."
        cd "$SCRIPT_DIR"
        if ! git pull; then
            error "git pull failed. Please resolve any conflicts or issues and run the script again."
        fi
        cd - > /dev/null
        info "Successfully pulled latest changes. The application will now be rebuilt."
    }

    show_menu_linux() {
        if [ ! -f "$dist_path/$package_name" ]; then
            # No existing build, so we just continue to the build process
            return
        fi

        info "An existing build was found."
        echo "What would you like to do?"
        echo "  1. Launch existing version"
        echo "  2. Rebuild the application"
        echo "  3. Check for Updates and Rebuild"
        read -p "> " -n 1 -r
        echo
        case "$REPLY" in
            1)
                launch_app_linux
                exit 0
                ;;
            2)
                info "Proceeding with rebuild..."
                # Let the script continue
                ;;
            3)
                update_app_linux # This function will run git pull
                # After update, the script will continue to the rebuild phase
                ;;
            *)
                warn "Invalid option. Aborting."
                exit 1
                ;;
        esac
    }

    ask_clean_build_linux() {
        read -p "Perform a clean build? (This is slower but can fix some issues) (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            do_clean=1
        fi
    }

    build_project_linux() {
        info "Building project... This may take a few minutes."
        cd "$project_dir"
        if [ "$do_clean" -eq 1 ]; then
            info "Cleaning previous build artifacts..."
            cargo clean || warn "cargo clean command failed, but continuing anyway."
        fi
        if ! cargo build --release; then
            error "Project build failed."
        fi
        cd "$SCRIPT_DIR"
        info "Project built successfully."
    }

    create_package_readme_linux() {
        info "Creating package README..."
        cat > "$1/README.txt" << EOL
==================================
Rust Simulation - Instructions
==================================
Thank you for downloading Rust Simulation!
To run the game, navigate into this directory and run the executable:
./${package_name}
If the game does not start, please ensure you have installed the necessary system
dependencies for your Linux distribution as mentioned in the main project README.
EOL
    }

    package_release_linux() {
        info "Packaging release..."
        mkdir -p "$dist_path"
        info "  - Copying executable..."
        cp "$project_dir/target/release/$package_name" "$dist_path/"
        info "  - Copying data files..."
        cp -r "$project_dir/data" "$dist_path/data"
        create_package_readme_linux "$dist_path"
        info "Creating .tar.gz archive..."
        local archive_name="${package_name}_v${project_version}_linux.tar.gz"
        if ! tar -czf "$dist_dir/$archive_name" -C "$dist_path" .; then
            error "Failed to create .tar.gz archive."
        fi
        info "Package created at $dist_dir/$archive_name"
    }

    # =========================================================================
    # Main Linux Execution Logic
    # =========================================================================
    show_menu_linux
    check_linux_dependencies
    get_project_version_linux
    ask_clean_build_linux
    install_rust_linux
    install_system_deps_linux
    build_project_linux
    package_release_linux
    launch_app_linux
}

run_windows() {
    info "Windows operating system detected."
    info "Handing off to Windows PowerShell script..."

    # Check if PowerShell is available
    if ! command -v powershell.exe &> /dev/null; then
        error "PowerShell is not found. Please run the 'run-windows.bat' script directly or install PowerShell."
    fi

    # Execute the PowerShell script
    # The -ExecutionPolicy Bypass is used to ensure the script can run on systems with restrictive policies.
    powershell.exe -ExecutionPolicy Bypass -File "$SCRIPT_DIR/setup-windows.ps1"
}

run_macos() {
    # --- OS Version Detection ---
    local product_name=$(sw_vers -productName)
    local product_version=$(sw_vers -productVersion)
    local build_version=$(sw_vers -buildVersion)
    info "macOS detected: $product_name $product_version (Build $build_version)"

    warn "macOS support is experimental."
    warn "This script will attempt to install Rust and build the project, but system dependency installation is not implemented."
    warn "Please ensure you have the necessary development tools (like Xcode Command Line Tools) installed."

    # For now, we can reuse the Linux logic since it's very similar.
    # A more mature script might have a dedicated run_macos function.
    run_linux
}

# =============================================================================
# Main Script Entry Point
# =============================================================================
main() {
    info "Starting the Rust Simulation launcher..."

    # --- Detect Operating System ---
    os_name="$(uname -s)"
    case "$os_name" in
        Linux*)
            run_linux
            ;;
        Darwin*)
            run_macos
            ;;
        CYGWIN*|MINGW*|MSYS*)
            run_windows
            ;;
        *)
            error "Unsupported operating system: $os_name"
            ;;
    esac

    info "Script finished."
}

# --- Execute main function ---
main "$@"
