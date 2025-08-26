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
LANG_CONFIG_FILE="$SCRIPT_DIR/.lang.cfg"

# --- State Variables ---
LANG="en"
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
    local msg_en="$1"
    local msg_ru="$2"
    if [ "$LANG" == "ru" ] && [ -n "$msg_ru" ]; then
        echo -e "${C_BLUE}INFO:${C_RESET} $msg_ru"
    else
        echo -e "${C_BLUE}INFO:${C_RESET} $msg_en"
    fi
}
warn() {
    local msg_en="$1"
    local msg_ru="$2"
    if [ "$LANG" == "ru" ] && [ -n "$msg_ru" ]; then
        echo -e "${C_YELLOW}WARN:${C_RESET} $msg_ru"
    else
        echo -e "${C_YELLOW}WARN:${C_RESET} $msg_en"
    fi
}
error() {
    local msg_en="$1"
    local msg_ru="$2"
    if [ "$LANG" == "ru" ] && [ -n "$msg_ru" ]; then
        echo -e "${C_RED}ERROR:${C_RESET} $msg_ru" >&2
    else
        echo -e "${C_RED}ERROR:${C_RESET} $msg_en" >&2
    fi
    exit 1
}

# --- Core Logic Functions ---

select_language() {
    if [ -f "$LANG_CONFIG_FILE" ]; then
        LANG=$(cat "$LANG_CONFIG_FILE")
        return
    fi

    clear
    echo "Choose your language / Выберите ваш язык:"
    echo " 1. English"
    echo " 2. Русский"
    read -p "> " -n 1 -r
    echo
    if [[ $REPLY =~ ^[2]$ ]]; then
        LANG="ru"
    else
        LANG="en" # Default to English
    fi
    echo "$LANG" > "$LANG_CONFIG_FILE"
}

check_dependencies() {
    info "Checking for required command-line tools..." "Проверка необходимых инструментов..."
    local missing_tool=0
    for tool in curl grep sed tar; do
        if ! command -v "$tool" &> /dev/null; then
            warn "Command '$tool' is not found, but is required." "Команда '$tool' не найдена, но она необходима."
            missing_tool=1
        fi
    done
    if [ "$missing_tool" -eq 1 ]; then
        error "Please install the missing tools and run the script again." "Пожалуйста, установите недостающие инструменты и запустите скрипт снова."
    fi
}

install_rust() {
    if command -v "cargo" &> /dev/null; then
        info "Rust is already installed." "Rust уже установлен."
        return
    fi

    warn "Rust (cargo) not found." "Rust (cargo) не найден."

    local prompt_en="Would you like to install it now using the official rustup script? (y/N) "
    local prompt_ru="Хотите установить его сейчас с помощью официального скрипта rustup? (y/N) "
    local prompt_text=$prompt_en
    [ "$LANG" == "ru" ] && prompt_text=$prompt_ru

    read -p "$prompt_text" -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Downloading and running rustup-init.sh..." "Загрузка и запуск rustup-init.sh..."
        if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
            error "Rust installation failed." "Установка Rust не удалась."
        fi
        source "$HOME/.cargo/env"
        info "Rust installed successfully." "Rust успешно установлен."
        warn "Please restart your terminal after this script finishes for the changes to take full effect." "Пожалуйста, перезапустите терминал после завершения этого скрипта, чтобы изменения вступили в силу."
    else
        error "Rust installation skipped. Cannot proceed." "Установка Rust пропущена. Невозможно продолжить."
    fi
}

install_system_deps() {
    info "Checking for system dependencies..." "Проверка системных зависимостей..."

    local os_id=""
    if [ -f /etc/os-release ]; then
        os_id=$(grep -oP '(?<=^ID=).+' /etc/os-release | tr -d '"')
    else
        warn "Could not determine Linux distribution. Cannot automatically check system dependencies." "Не удалось определить дистрибутив Linux. Невозможно проверить системные зависимости."
        return
    fi

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
            warn "Unsupported Linux distribution '$os_id'. Cannot check system dependencies." "Неподдерживаемый дистрибутив Linux '$os_id'. Невозможно проверить системные зависимости."
            return ;;
    esac

    for pkg in $pkgs_needed; do
        if ! $check_cmd "$pkg" &> /dev/null; then
            pkgs_to_install="$pkgs_to_install $pkg"
        fi
    done

    if [ -n "$pkgs_to_install" ]; then
        warn "The following system dependencies are required:$pkgs_to_install" "Требуются следующие системные зависимости:$pkgs_to_install"

        local prompt_en="Would you like to install them now? (This will use sudo) (y/N) "
        local prompt_ru="Хотите установить их сейчас? (Будет использовано sudo) (y/N) "
        local prompt_text=$prompt_en
        [ "$LANG" == "ru" ] && prompt_text=$prompt_ru

        read -p "$prompt_text" -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if ! $install_cmd $pkgs_to_install; then
                error "Failed to install system dependencies. Please try to install them manually." "Не удалось установить системные зависимости. Пожалуйста, попробуйте установить их вручную."
            fi
            info "System dependencies installed successfully." "Системные зависимости успешно установлены."
        else
            warn "Installation of system dependencies skipped. The build may fail." "Установка системных зависимостей пропущена. Сборка может завершиться неудачно."
        fi
    else
        info "All required system dependencies are already installed." "Все необходимые системные зависимости уже установлены."
    fi
}

get_project_version() {
    info "Getting project version..." "Получение версии проекта..."
    if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
        error "Cargo.toml not found in $PROJECT_DIR" "Cargo.toml не найден в $PROJECT_DIR"
    fi
    local version_line=$(grep "^version" "$PROJECT_DIR/Cargo.toml" | head -n 1)
    PROJECT_VERSION=$(echo "$version_line" | sed -E 's/version\s*=\s*"([^"]+)"/\1/')
}

ask_clean_build() {
    local prompt_en="Perform a clean build? (This is slower but can fix some issues) (y/N) "
    local prompt_ru="Выполнить чистую сборку? (Это дольше, но может исправить некоторые проблемы) (y/N) "
    local prompt_text=$prompt_en
    [ "$LANG" == "ru" ] && prompt_text=$prompt_ru

    read -p "$prompt_text" -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        DO_CLEAN=1
    fi
}

build_project() {
    info "Building project... This may take a few minutes." "Сборка проекта... Это может занять несколько минут."

    cd "$PROJECT_DIR"

    if [ "$DO_CLEAN" -eq 1 ]; then
        info "Cleaning previous build artifacts..." "Очистка предыдущих артефактов сборки..."
        if ! cargo clean; then
            warn "cargo clean command failed, but continuing anyway." "Команда cargo clean не удалась, но продолжим."
        fi
    fi

    if ! cargo build --release; then
        error "Project build failed." "Сборка проекта не удалась."
    fi

    cd "$SCRIPT_DIR"
    info "Project built successfully." "Проект успешно собран."
}

create_package_readme() {
    info "Creating package README..." "Создание README для пакета..."
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

---

==================================
Rust Simulation - Инструкции
==================================

Спасибо за загрузку Rust Simulation!

Чтобы запустить игру, перейдите в этот каталог и запустите исполняемый файл:
./${PACKAGE_NAME}

Если игра не запускается, пожалуйста, убедитесь, что вы установили необходимые
системные зависимости для вашего дистрибутива Linux, как указано в основном
README проекта.
EOL
}

package_release() {
    info "Packaging release..." "Упаковка релиза..."

    # Create directories
    mkdir -p "$DIST_PATH"

    # Copy files
    info "Copying files..." "Копирование файлов..."
    cp "$PROJECT_DIR/target/release/$PACKAGE_NAME" "$DIST_PATH/"
    cp -r "$PROJECT_DIR/data" "$DIST_PATH/data"

    # Create README
    create_package_readme "$DIST_PATH"

    # Create Tarball
    info "Creating .tar.gz archive..." "Создание .tar.gz архива..."
    local archive_name="${PACKAGE_NAME}_v${PROJECT_VERSION}_linux.tar.gz"
    if ! tar -czf "$DIST_DIR/$archive_name" -C "$DIST_PATH" .; then
        error "Failed to create .tar.gz archive." "Не удалось создать .tar.gz архив."
    fi

    info "Package created at $DIST_DIR/$archive_name" "Пакет создан в $DIST_DIR/$archive_name"
}

launch_app() {
    info "Launching application..." "Запуск приложения..."

    cd "$DIST_PATH"
    "./$PACKAGE_NAME"
}

# =============================================================================
# Script Entry Point
# =============================================================================
main() {
    select_language
    clear

    info "Rust Simulation Linux Launcher" "Лончер Rust Simulation для Linux"
    echo "===================================="

    check_dependencies
    get_project_version
    ask_clean_build

    install_rust
    install_system_deps

    build_project
    package_release
    launch_app

    info "Script finished." "Скрипт завершен."
}

main
