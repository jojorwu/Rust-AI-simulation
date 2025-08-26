@echo off
setlocal enabledelayedexpansion

:: =============================================================================
:: Main Script Logic
:: =============================================================================

call :initialize
call :check_rust
if !errorlevel! neq 0 (
    call :install_rust
    if !errorlevel! neq 0 (
        echo !msg_rust_install_failed!
        goto :end
    )
    call :post_rust_install_message
    goto :end
)
call :build_project
if !errorlevel! neq 0 (
    goto :end
)
call :package_app
if !errorlevel! neq 0 (
    goto :end
)
call :launch_app
goto :end

:: =============================================================================
:: Subroutines
:: =============================================================================

:initialize
    :: Set language, falling back to selection prompt if not configured
    if exist "lang.cfg" (
        set /p lang=<lang.cfg
    ) else (
        echo Choose your language / Выберите ваш язык:
        echo  1. English
        echo  2. Русский
        choice /c 12 /n /m ">"
        if errorlevel 2 (set lang=RU) else (set lang=EN)
        echo !lang! > lang.cfg
        echo.
    )

    :: Set console code page for Russian
    if "!lang!"=="RU" (
        chcp 65001 > nul
    )

    call :set_strings
    cls
    echo !msg_welcome!
    echo.
    exit /b 0

:set_strings
    if "!lang!"=="EN" (
        set "msg_welcome=Welcome to the Rust Simulation setup script."
        set "msg_cargo_found=Cargo found."
        set "msg_cargo_not_found=Cargo not found. Attempting to download the Rust installer."
        set "msg_download_failed=! AUTOMATIC DOWNLOAD FAILED. Please download the Rust installer manually."
        set "msg_download_link=1. Download from: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        set "msg_download_save=2. Save 'rustup-init.exe' in the same folder as this script."
        set "msg_download_return=3. Return to this window when the download is complete."
        set "msg_file_not_found='rustup-init.exe' still not found. Please try again."
        set "msg_starting_installer=Starting the Rust installer..."
        set "msg_rust_install_failed=Rust installation failed or was cancelled."
        set "msg_post_install_title=Rust has been installed."
        set "msg_post_install_important=IMPORTANT: You must restart your command prompt or terminal for the changes to take effect."
        set "msg_post_install_restart=Please close this window and run this script again."
        set "msg_building=Building the project... This may take a few minutes."
        set "msg_build_failed=Build failed. Please check for errors in the output above."
        set "msg_build_verify_failed=ERROR: Build seemed to succeed, but the executable was not found."
        set "msg_packaging=Build successful. Packaging the application..."
        set "msg_copy_failed=ERROR: Failed to copy the executable to the 'dist/windows' folder."
        set "msg_packaged_successfully=Application packaged successfully!"
        set "msg_dist_location=The distributable is in the 'dist/windows' directory."
        set "msg_launching=Launching the application..."
    )
    if "!lang!"=="RU" (
        set "msg_welcome=Добро пожаловать в скрипт установки Rust Simulation."
        set "msg_cargo_found=Cargo найден."
        set "msg_cargo_not_found=Cargo не найден. Попытка загрузить установщик Rust."
        set "msg_download_failed=! АВТОМАТИЧЕСКАЯ ЗАГРУЗКА НЕ УДАЛАСЬ. Пожалуйста, загрузите установщик Rust вручную."
        set "msg_download_link=1. Скачайте файл с: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        set "msg_download_save=2. Сохраните 'rustup-init.exe' в ту же папку, где находится этот скрипт."
        set "msg_download_return=3. Когда скачивание завершится, вернитесь в это окно."
        set "msg_file_not_found=Файл 'rustup-init.exe' все еще не найден. Пожалуйста, попробуйте снова."
        set "msg_starting_installer=Запуск установщика Rust..."
        set "msg_rust_install_failed=Установка Rust не удалась или была отменена."
        set "msg_post_install_title=Rust был установлен."
        set "msg_post_install_important=ВАЖНО: Вы должны перезапустить командную строку или терминал, чтобы изменения вступили в силу."
        set "msg_post_install_restart=Пожалуйста, закройте это окно и запустите скрипт снова."
        set "msg_building=Сборка проекта... Это может занять несколько минут."
        set "msg_build_failed=Сборка не удалась. Пожалуйста, проверьте наличие ошибок."
        set "msg_build_verify_failed=ОШИБКА: Сборка вроде бы прошла успешно, но исполняемый файл не найден."
        set "msg_packaging=Сборка прошла успешно. Упаковка приложения..."
        set "msg_copy_failed=ОШИБКА: Не удалось скопировать исполняемый файл в папку 'dist/windows'."
        set "msg_packaged_successfully=Приложение успешно упаковано!"
        set "msg_dist_location=Готовое приложение находится в папке 'dist/windows'."
        set "msg_launching=Запускаю приложение..."
    )
    exit /b 0

:check_rust
    cargo --version >nul 2>&1
    if %errorlevel% equ 0 (
        echo !msg_cargo_found!
        exit /b 0
    ) else (
        exit /b 1
    )

:install_rust
    echo !msg_cargo_not_found!
    echo.
    powershell -Command "Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe" >nul 2>&1
    if !errorlevel! neq 0 (
        echo.
        echo ====================================================================
        echo !msg_download_failed!
        echo.
        echo !msg_download_link!
        echo !msg_download_save!
        echo !msg_download_return!
        echo ====================================================================
        echo.
        pause
        if not exist "rustup-init.exe" (
            echo !msg_file_not_found!
            exit /b 1
        )
    )
    echo !msg_starting_installer!
    start /wait rustup-init.exe --default-toolchain stable -y
    if exist "rustup-init.exe" (
        del "rustup-init.exe"
    )
    exit /b 0

:post_rust_install_message
    echo.
    echo ====================================================================
    echo  !msg_post_install_title!
    echo  !msg_post_install_important!
    echo.
    echo  !msg_post_install_restart!
    echo ====================================================================
    exit /b 0

:build_project
    echo !msg_building!
    echo.
    cd rust_simulation
    cargo build --release > nul
    if !errorlevel! neq 0 (
        echo !msg_build_failed!
        cd ..
        exit /b 1
    )
    if not exist "target\release\rust_simulation.exe" (
        echo !msg_build_verify_failed!
        cd ..
        exit /b 1
    )
    cd ..
    exit /b 0

:package_app
    echo !msg_packaging!
    if not exist "dist" mkdir "dist"
    if not exist "dist\windows" mkdir "dist\windows"
    copy "rust_simulation\target\release\rust_simulation.exe" "dist\windows\" > nul
    xcopy "rust_simulation\data" "dist\windows\data\" /E /I /Y /Q
    if not exist "dist\windows\rust_simulation.exe" (
        echo !msg_copy_failed!
        exit /b 1
    )
    exit /b 0

:launch_app
    echo.
    echo ======================================================
    echo  !msg_packaged_successfully!
    echo  !msg_dist_location!
    echo  !msg_launching!
    echo ======================================================
    echo.
    cd "dist\windows"
    start rust_simulation.exe
    exit /b 0

:end
    echo.
    pause
    endlocal
