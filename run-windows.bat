@echo off
setlocal enabledelayedexpansion

:: =============================================================================
:: Configuration & Strings
:: =============================================================================
set "msg_welcome=Welcome to the Rust Simulation setup script."
set "msg_version_found=Project version found:"
set "msg_existing_build_found=An existing build was found. What would you like to do?"
set "msg_clean_build_prompt=Perform a clean build? (This is slower but can fix some issues)"
set "msg_cleaning=Cleaning previous build artifacts..."
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
set "msg_build_failed_help=Note: If the error mentions 'linker' or 'LNK' errors, you may need to install the 'C++ build tools' from the Visual Studio Installer."
set "msg_build_verify_failed=ERROR: Build seemed to succeed, but the executable was not found."
set "msg_packaging=Build successful. Packaging the application..."
set "msg_copy_failed=ERROR: Failed to copy the executable to the 'dist/windows' folder."
set "msg_bundling_redist=Bundling Microsoft VC++ Redistributable..."
set "msg_bundling_redist_failed=Warning: Failed to download VC++ Redistributable. The application may not run on PCs without development tools installed."
set "msg_creating_zip=Creating ZIP archive..."
set "msg_creating_zip_failed=Warning: Failed to create ZIP archive."
set "msg_packaged_successfully=Application packaged successfully!"
set "msg_dist_location=The distributable is in the 'dist/windows' directory and 'dist/rust_simulation_v!project_version!.zip'."
set "msg_launching=Launching the application..."

:: =============================================================================
:: Main Script Logic
:: =============================================================================
cls
echo !msg_welcome!
echo.

call :check_powershell
if !errorlevel! neq 0 ( goto :end )

call :get_version

call :check_existing_build
if "!rebuild!"=="0" (
    call :launch_app
    goto :end
)

call :ask_clean_build
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
if !errorlevel! neq 0 ( goto :end )

call :package_app
if !errorlevel! neq 0 ( goto :end )

call :bundle_redist
if !errorlevel! neq 0 ( goto :end )

call :create_zip
if !errorlevel! neq 0 ( goto :end )

call :launch_app
goto :end

:: =============================================================================
:: Subroutines
:: =============================================================================

:check_powershell
	where powershell >nul 2>&1
	if !errorlevel! neq 0 (
		echo ERROR: PowerShell is not found on this system.
		echo This script requires PowerShell to download and package files.
		echo Please install PowerShell or run it from a modern Windows system.
		exit /b 1
	)
	exit /b 0

:get_version
    set "project_version="
    set "toml_path=%~dp0rust_simulation\Cargo.toml"
    for /f "usebackq tokens=*" %%i in (`powershell -Command "$in_package_section = $false; foreach ($line in (Get-Content -Path '%toml_path%')) { $trimmed_line = $line.Trim(); if ($trimmed_line -eq '[package]') { $in_package_section = $true; continue; }; if ($trimmed_line.StartsWith('[') -and $trimmed_line.EndsWith(']')) { $in_package_section = $false; }; if ($in_package_section -and $trimmed_line -match '^version\s*=\s*') { $version = ($trimmed_line.Split('=')[1]).Trim().Trim('\"'); Write-Output $version; break; } } "`) do (
        set "project_version=%%i"
    )
    if "!project_version!"=="" (
        set "project_version=unknown"
    )
    echo !msg_version_found! !project_version!
    exit /b 0

:check_existing_build
    set "rebuild=1"
    if exist "%~dp0dist\windows\rust_simulation.exe" (
        echo.
        echo !msg_existing_build_found!
        echo  1. Launch the existing version
        echo  2. Rebuild the application
        choice /c 12 /n /m "> "
        if errorlevel 2 (
            set "rebuild=1"
        ) else (
            set "rebuild=0"
        )
    )
    exit /b 0

:ask_clean_build
    set "do_clean=0"
    echo.
    echo !msg_clean_build_prompt!
    echo (Y/N)
    choice /c YN /n
    if errorlevel 2 (exit /b 0) else (set "do_clean=1")
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
    set "download_log=%~dp0rustup_download.log"
    if exist "!download_log!" del "!download_log!" >nul 2>&1

    powershell -Command "Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile '%~dp0rustup-init.exe'" > "!download_log!" 2>&1
    if !errorlevel! neq 0 (
        echo.
        echo ====================================================================
        echo !msg_download_failed!
        echo.
        echo --- PowerShell Log ---
        type "!download_log!"
        echo ----------------------
        echo.
        echo !msg_download_link!
        echo !msg_download_save!
        echo !msg_download_return!
        echo ====================================================================
        echo.
        pause
        if not exist "%~dp0rustup-init.exe" (
            echo !msg_file_not_found!
            exit /b 1
        )
    )
    if exist "!download_log!" del "!download_log!" >nul 2>&1
    echo !msg_starting_installer!
    start /wait "%~dp0rustup-init.exe" --default-toolchain stable -y
    if exist "%~dp0rustup-init.exe" (
        del "%~dp0rustup-init.exe"
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
    set "build_log=%~dp0build.log"
    if exist "!build_log!" del "!build_log!" >nul 2>&1

    cd "%~dp0rust_simulation" || exit /b 1
    if "!do_clean!"=="1" (
        echo !msg_cleaning!
        cargo clean > nul
    )

    cargo build --release > "!build_log!" 2>&1
    if !errorlevel! neq 0 (
        echo.
        echo ====================================================================
        echo !msg_build_failed!
        echo !msg_build_failed_help!
        echo ====================================================================
        echo.
        echo Build log:
        type "!build_log!"
        echo.
        cd "%~dp0"
        exit /b 1
    )

    if not exist "target\release\rust_simulation.exe" (
        echo !msg_build_verify_failed!
        cd "%~dp0"
        exit /b 1
    )

    cd "%~dp0"
    if exist "!build_log!" del "!build_log!" >nul 2>&1
    exit /b 0

:package_app
    echo !msg_packaging!

    if not exist "%~dp0dist" (
        mkdir "%~dp0dist"
        if !errorlevel! neq 0 (
            echo ERROR: Failed to create 'dist' directory.
            exit /b 1
        )
    )

    if not exist "%~dp0dist\windows" (
        mkdir "%~dp0dist\windows"
        if !errorlevel! neq 0 (
            echo ERROR: Failed to create 'dist\windows' directory.
            exit /b 1
        )
    )

    echo   - Copying executable...
    copy "%~dp0rust_simulation\target\release\rust_simulation.exe" "%~dp0dist\windows\" > nul
    if !errorlevel! neq 0 (
        echo ERROR: Failed to copy the main executable. Was the build successful?
        exit /b 1
    )

    echo   - Copying data files...
    xcopy "%~dp0rust_simulation\data" "%~dp0dist\windows\data\" /E /I /Y /Q > nul
    if !errorlevel! neq 0 (
        echo ERROR: Failed to copy the 'data' directory.
        exit /b 1
    )

    if not exist "%~dp0dist\windows\rust_simulation.exe" (
        echo !msg_copy_failed!
        exit /b 1
    )

    call :create_package_readme
    if !errorlevel! neq 0 (
        echo ERROR: Failed to create the package README.txt file.
        exit /b 1
    )

    exit /b 0

:create_package_readme
    (
        echo ==================================
        echo Rust Simulation - Instructions
        echo ==================================
        echo.
        echo Thank you for downloading Rust Simulation!
        echo.
        echo To run the game, simply run the `rust_simulation.exe` file.
        echo.
        echo If the game does not start and you see an error about a missing DLL ^(like VCRUNTIME140.dll^), please run the included `vc_redist.x64.exe` installer first. This is the official Microsoft C++ library installer and is required by the game engine.
    ) > "%~dp0dist\windows\README.txt"
    exit /b 0

:bundle_redist
    echo !msg_bundling_redist!
    set "redist_log=%~dp0redist_download.log"
    if exist "!redist_log!" del "!redist_log!" >nul 2>&1

    cd "%~dp0dist\windows" || exit /b 1
    powershell -Command "Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vc_redist.x64.exe' -OutFile 'vc_redist.x64.exe'" > "!redist_log!" 2>&1
    if !errorlevel! neq 0 (
        echo !msg_bundling_redist_failed!
        echo.
        echo Download log:
        type "!redist_log!"
    )
    cd "%~dp0"
    if exist "!redist_log!" del "!redist_log!" >nul 2>&1
    exit /b 0

:create_zip
    echo !msg_creating_zip!
    set "zip_log=%~dp0zip.log"
    if exist "!zip_log!" del "!zip_log!" >nul 2>&1
    set "zip_path=%~dp0dist\rust_simulation_v!project_version!.zip"
    set "source_path=%~dp0dist\windows"
    powershell -Command "Compress-Archive -Path '%source_path%\*' -DestinationPath '%zip_path%' -Force" > "!zip_log!" 2>&1
    if !errorlevel! neq 0 (
        echo !msg_creating_zip_failed!
        echo.
        echo ZIP log:
        type "!zip_log!"
    )
    if exist "!zip_log!" del "!zip_log!" >nul 2>&1
    exit /b 0

:launch_app
    echo.
    echo ======================================================
    echo  !msg_packaged_successfully!
    echo  !msg_dist_location!
    echo  !msg_launching!
    echo ======================================================
    echo.
    cd "%~dp0dist\windows" && start rust_simulation.exe
    exit /b 0

:end
    echo.
    pause
    endlocal
