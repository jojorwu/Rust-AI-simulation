@echo off
setlocal enabledelayedexpansion

:: =============================================================================
:: Main Script Logic
:: =============================================================================

call :check_powershell
if !errorlevel! neq 0 (
	goto :end
)

call :check_font
if !errorlevel! neq 0 (
	call :show_font_instructions
	goto :end
)

call :initialize
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
if !errorlevel! neq 0 (
    goto :end
)
call :package_app
if !errorlevel! neq 0 (
    goto :end
)
call :bundle_redist
if !errorlevel! neq 0 (
    goto :end
)
call :create_zip
if !errorlevel! neq 0 (
    goto :end
)
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
		echo.
		echo ОШИБКА: PowerShell не найден в этой системе.
		echo Этому скрипту требуется PowerShell для загрузки и упаковки файлов.
		echo Пожалуйста, установите PowerShell или запустите скрипт в современной системе Windows.
		exit /b 1
	)
	exit /b 0

:check_font
	chcp 65001 > nul
	cls
	echo ====================================================================
	echo Font Check / Проверка шрифта
	echo ====================================================================
	echo.
	echo This script uses Unicode characters for messages. Please check if the
	echo Russian text below displays correctly.
	echo.
	echo --> Тест / Test <--
	echo.
	echo Can you see the Russian text correctly ^(and not as '?????'^)?
	echo (Y/N)
	choice /c YN /n
	if errorlevel 2 (exit /b 1) else (exit /b 0)

:show_font_instructions
	cls
	echo ====================================================================
	echo Font Configuration Needed / Требуется настройка шрифта
	echo ====================================================================
	echo.
	echo Your console font does not support Unicode characters, which is
	echo needed for the Russian language prompts.
	echo.
	echo Please follow these steps to fix it:
	echo 1. Right-click the title bar of this window.
	echo 2. Select 'Properties'.
	echo 3. Go to the 'Font' tab.
	echo 4. Choose a modern font like 'Consolas' or 'Lucida Console'.
	echo    (Do NOT use 'Raster Fonts' or 'Terminal'.)
	echo 5. Click 'OK' to save.
	echo 6. Close this window and run the script again.
	echo.
	echo ---
	echo.
	echo Вашему шрифту в консоли не хватает поддержки Юникода,
	echo которая необходима для отображения сообщений на русском языке.
	echo.
	echo Пожалуйста, следуйте этим инструкциям:
	echo 1. Нажмите правой кнопкой мыши на заголовок этого окна.
	echo 2. Выберите 'Свойства'.
	echo 3. Перейдите на вкладку 'Шрифт'.
	echo 4. Выберите современный шрифт, например, 'Consolas' или 'Lucida Console'.
	echo    (Не используйте 'Растровые шрифты' или 'Terminal'.)
	echo 5. Нажмите 'OK', чтобы сохранить.
	echo 6. Закройте это окно и запустите скрипт еще раз.
	exit /b 1

:initialize
    :: Set language, falling back to selection prompt if not configured
    if exist "%~dp0lang.cfg" (
        set /p lang=<"%~dp0lang.cfg"
    ) else (
        echo Choose your language / Выберите ваш язык:
        echo  1. English
        echo  2. Русский
        choice /c 12 /n /m ">"
        if errorlevel 2 (set lang=RU) else (set lang=EN)
        echo !lang! > "%~dp0lang.cfg"
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

:get_version
    set "project_version="
    set "toml_path=%~dp0rust_simulation\Cargo.toml"
    for /f "usebackq tokens=*" %%i in (`powershell -Command "(Get-Content -Path '%toml_path%' | Select-String -Pattern 'version\s*=').Line.Split('=')[1].Trim().Trim('\"')"`) do (
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

:set_strings
    if "!lang!"=="EN" (
        set "msg_welcome=Welcome to the Rust Simulation setup script."
        set "msg_version_found=Project version found:"
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
        set "msg_existing_build_found=An existing build was found. What would you like to do?"
    )
    if "!lang!"=="RU" (
        set "msg_welcome=Добро пожаловать в скрипт установки Rust Simulation."
        set "msg_version_found=Найдена версия проекта:"
        set "msg_clean_build_prompt=Выполнить чистую сборку? (Это дольше, но может исправить некоторые проблемы)"
        set "msg_cleaning=Очистка предыдущих артефактов сборки..."
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
        set "msg_build_failed_help=Примечание: Если ошибка упоминает 'linker' или 'LNK', вам может потребоваться установить 'C++ build tools' через Visual Studio Installer."
        set "msg_build_verify_failed=ОШИБКА: Сборка вроде бы прошла успешно, но исполняемый файл не найден."
        set "msg_packaging=Сборка прошла успешно. Упаковка приложения..."
        set "msg_copy_failed=ОШИБКА: Не удалось скопировать исполняемый файл в папку 'dist/windows'."
        set "msg_bundling_redist=Добавление Microsoft VC++ Redistributable..."
        set "msg_bundling_redist_failed=Предупреждение: Не удалось скачать VC++ Redistributable. Приложение может не запуститься на ПК без установленных средств разработки."
        set "msg_creating_zip=Создание ZIP архива..."
        set "msg_creating_zip_failed=Предупреждение: Не удалось создать ZIP архив."
        set "msg_packaged_successfully=Приложение успешно упаковано!"
        set "msg_dist_location=Готовое приложение находится в папке 'dist/windows' и в 'dist/rust_simulation_v!project_version!.zip'."
        set "msg_launching=Запускаю приложение..."
        set "msg_existing_build_found=Найдена существующая сборка. Что вы хотите сделать?"
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
    if not exist "%~dp0dist" mkdir "%~dp0dist"
    if not exist "%~dp0dist\windows" mkdir "%~dp0dist\windows"
    copy "%~dp0rust_simulation\target\release\rust_simulation.exe" "%~dp0dist\windows\" > nul
    xcopy "%~dp0rust_simulation\data" "%~dp0dist\windows\data\" /E /I /Y /Q
    if not exist "%~dp0dist\windows\rust_simulation.exe" (
        echo !msg_copy_failed!
        exit /b 1
    )
    call :create_package_readme
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
        echo.
        echo ---
        echo.
        echo ==================================
        echo Rust Simulation - Инструкции
        echo ==================================
        echo.
        echo Спасибо за загрузку Rust Simulation!
        echo.
        echo Чтобы запустить игру, просто запустите файл `rust_simulation.exe`.
        echo.
        echo Если игра не запускается и вы видите ошибку об отсутствующей DLL ^(например, VCRUNTIME140.dll^), пожалуйста, сначала запустите установщик `vc_redist.x64.exe`, который находится в этой же папке. Это официальный установщик библиотек Microsoft C++, который требуется для игрового движка.
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
