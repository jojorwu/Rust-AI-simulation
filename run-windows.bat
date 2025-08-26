@echo off
:: Set the code page to UTF-8 to support Cyrillic characters
chcp 65001 > nul
setlocal

:: Check if rustup-init.exe exists from a previous failed attempt
if exist "rustup-init.exe" (
    goto :run_installer
)

:: Check if cargo is installed
cargo --version >nul 2>&1
if %errorlevel% equ 0 (
    goto :build_project
)

:install_rust
echo Cargo не найден. Этот скрипт попытается загрузить и запустить установщик Rust.
echo.

:: Attempt to download the installer
powershell -Command "Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe"
if %errorlevel% neq 0 (
    echo.
    echo.
    echo ====================================================================
    echo ! Автоматическая загрузка не удалась. Пожалуйста, загрузите установщик Rust вручную.
    echo.
    echo 1. Скачайте файл с: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
    echo 2. Сохраните 'rustup-init.exe' в ту же папку, где находится этот скрипт.
    echo 3. Когда скачивание завершится, вернитесь в это окно.
    echo ====================================================================
    echo.
    pause
    if not exist "rustup-init.exe" (
        echo Файл 'rustup-init.exe' все еще не найден. Пожалуйста, попробуйте снова.
        pause
        exit /b 1
    )
)

:run_installer
echo.
echo Запуск установщика Rust...
start /wait rustup-init.exe --default-toolchain stable -y

echo.
echo ====================================================================
echo  Rust был установлен.
echo  ВАЖНО: Вы должны перезапустить командную строку или терминал,
echo  чтобы изменения вступили в силу.
echo.
echo  Пожалуйста, закройте это окно и запустите скрипт снова.
echo ====================================================================
echo.
pause
exit /b 0


:build_project
echo Cargo найден. Сборка проекта...
echo Это может занять несколько минут.
echo.

:: Navigate to the rust_simulation directory and build
cd rust_simulation
cargo build --release

if %errorlevel% neq 0 (
    echo Сборка не удалась. Пожалуйста, проверьте наличие ошибок.
    pause
    exit /b 1
)

:: Verify that the executable was created
if not exist "target\release\rust_simulation.exe" (
    echo.
    echo ОШИБКА: Сборка вроде бы прошла успешно, но файл 'target\release\rust_simulation.exe' не найден.
    echo Что-то пошло не так во время компиляции.
    echo.
    pause
    exit /b 1
)

echo Сборка прошла успешно. Упаковка приложения...
echo.

:: Navigate back to the root
cd ..

:: Create distribution directory
if not exist "dist" mkdir "dist"
if not exist "dist\windows" mkdir "dist\windows"

:: Copy executable
copy "rust_simulation\target\release\rust_simulation.exe" "dist\windows\"

:: Copy data files
xcopy "rust_simulation\data" "dist\windows\data\" /E /I /Y

:: Verify that the executable was copied
if not exist "dist\windows\rust_simulation.exe" (
    echo.
    echo ОШИБКА: Не удалось скопировать 'rust_simulation.exe' в папку 'dist\windows'.
    echo Пожалуйста, проверьте, есть ли у вас права на запись в этой папке.
    echo.
    pause
    exit /b 1
)

echo.
echo ======================================================
echo  Приложение успешно упаковано!
echo  Готовое приложение находится в папке 'dist\windows'.
echo  Запускаю приложение...
echo ======================================================
echo.

:: Run the application
cd "dist\windows"
start rust_simulation.exe

endlocal
