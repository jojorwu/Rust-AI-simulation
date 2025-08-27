#!/bin/bash

# This script automates the process of building, packaging, and running the Rust Simulation.
# It includes dependency checking, installation helpers, and release packaging.

# Stop on any error
set -e

# =============================================================================
# Global Variables and Configuration
# =============================================================================
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_DIR="$SCRIPT_DIR/rust_simulation"
DIST_DIR="$SCRIPT_DIR/dist"
DIST_PATH="$DIST_DIR/linux"

# --- State Variables ---
DO_CLEAN=0
PROJECT_VERSION="unknown"
PACKAGE_NAME="rust_simulation"

# Required packages for different Linux distributions
DEPS_DEBIAN="build-essential libasound2-dev libudev-dev"
DEPS_FEDORA="alsa-lib-devel libudev-devel systemd-devel"
DEPS_ARCH="base-devel alsa-lib"

# --- Color Codes ---
C_RESET='\033[0m'
C_RED='\033[0;31m'
C_GREEN='\033[0;32m'
C_YELLOW='\033[0;33m'
C_BLUE='\033[0;34m'

# =============================================================================
# Subroutines
# =============================================================================

# --- Logging Helpers ---
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

# --- Core Logic Functions ---

check_dependencies() {
    info "Checking for required command-line tools..."
    local missing_tool=0
    for tool in curl grep sed tar awk; do
        if ! command -v "$tool" &> /dev/null; then
            warn "Command '$tool' is not found, but is required."
            missing_tool=1
        fi
    done
    if [ "$missing_tool" -eq 1 ]; then
        error "Please install the missing tools and run the script again."
    fi
}

install_rust() {
    if command -v "cargo" &> /dev/null; then
        info "Rust is already installed."
        return
    fi

    warn "Rust (cargo) not found."
    read -p "Would you like to install it now using the official rustup script? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Downloading and running rustup-init.sh..."
        if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
            error "Rust installation failed."
        fi
        source "$HOME/.cargo/env"
        info "Rust installed successfully."
        warn "Please restart your terminal after this script finishes for the changes to take full effect."
    else
        error "Rust installation skipped. Cannot proceed."
    fi
}

install_system_deps() {
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
            pkgs_needed=$DEPS_DEBIAN; check_cmd="dpkg -s"; install_cmd="sudo apt-get install -y" ;;
        "fedora")
            pkgs_needed=$DEPS_FEDORA; check_cmd="rpm -q"; install_cmd="sudo dnf install -y" ;;
        "arch")
            pkgs_needed=$DEPS_ARCH; check_cmd="pacman -Qs"; install_cmd="sudo pacman -S --noconfirm" ;;
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

get_project_version() {
    info "Getting project version..."
    if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
        error "Cargo.toml not found in $PROJECT_DIR"
    fi

    local version=$(awk '
        BEGIN { in_package = 0 }
        /^\s*\[package\]\s*$/ { in_package = 1; next }
        /^\s*\[.*\]\s*$/ { in_package = 0 }
        in_package && /^\s*version\s*=/ {
            match($0, /"(.*)"/);
            if (RSTART) {
                print substr($0, RSTART+1, RLENGTH-2);
                exit;
            }
        }
    ' "$PROJECT_DIR/Cargo.toml")

    if [ -n "$version" ]; then
        PROJECT_VERSION="$version"
    else
        PROJECT_VERSION="unknown"
        warn "Could not reliably determine project version. Defaulting to 'unknown'."
    fi
}

check_existing_build() {
    if [ -f "$DIST_PATH/$PACKAGE_NAME" ]; then
        info "An existing build was found."
        read -p "What would you like to do? (1. Launch existing version, 2. Rebuild the application) > " -n 1 -r
        echo
        if [[ $REPLY =~ ^[1]$ ]]; then
            launch_app
            exit 0
        fi
    fi
}

ask_clean_build() {
    read -p "Perform a clean build? (This is slower but can fix some issues) (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        DO_CLEAN=1
    fi
}

build_project() {
    info "Building project... This may take a few minutes."
    cd "$PROJECT_DIR"

    if [ "$DO_CLEAN" -eq 1 ]; then
        info "Cleaning previous build artifacts..."
        if ! cargo clean; then
            warn "cargo clean command failed, but continuing anyway."
        fi
    fi

    if ! cargo build --release; then
        error "Project build failed."
    fi

    cd "$SCRIPT_DIR"
    info "Project built successfully."
}

create_package_readme() {
    info "Creating package README..."
    local readme_path="$1/README.txt"

    cat > "$readme_path" << EOL
==================================
Rust Simulation - Instructions
==================================

Thank you for downloading Rust Simulation!

To run the game, navigate into this directory and run the executable:
./${PACKAGE_NAME}

If the game does not start, please ensure you have installed the necessary system
dependencies for your Linux distribution as mentioned in the main project README.
EOL
}

package_release() {
    info "Packaging release..."
    mkdir -p "$DIST_PATH"

    info "Copying files..."
    cp "$PROJECT_DIR/target/release/$PACKAGE_NAME" "$DIST_PATH/"
    cp -r "$PROJECT_DIR/data" "$DIST_PATH/data"

    create_package_readme "$DIST_PATH"

    info "Creating .tar.gz archive..."
    local archive_name="${PACKAGE_NAME}_v${PROJECT_VERSION}_linux.tar.gz"
    if ! tar -czf "$DIST_DIR/$archive_name" -C "$DIST_PATH" .; then
        error "Failed to create .tar.gz archive."
    fi

    info "Package created at $DIST_DIR/$archive_name"
}

launch_app() {
    info "Launching application..."
    cd "$DIST_PATH"
    "./$PACKAGE_NAME"
}

# =============================================================================
# Script Entry Point
# =============================================================================
main() {
    clear
    info "Rust Simulation Linux Launcher"
    echo "===================================="

    check_existing_build

    check_dependencies
    get_project_version
    ask_clean_build

    install_rust
    install_system_deps

    build_project
    package_release
    launch_app

    info "Script finished."
}

main
