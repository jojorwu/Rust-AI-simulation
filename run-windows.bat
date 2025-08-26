@echo off
setlocal

:: Check if cargo is installed
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Cargo is not found. This script will attempt to download and run the Rust installer.
    echo.
    powershell -Command "Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe"
    if %errorlevel% neq 0 (
        echo Failed to download the Rust installer. Please install it manually from https://www.rust-lang.org/tools/install
        pause
        exit /b 1
    )

    echo Starting the Rust installer...
    start /wait rustup-init.exe --default-toolchain stable -y

    echo.
    echo ====================================================================
    echo  Rust has been installed.
    echo  IMPORTANT: You must restart your command prompt or terminal
    echo  for the changes to take effect.
    echo  Please close this window and run this script again.
    echo ====================================================================
    echo.
    pause
    exit /b 0
)

echo Cargo found. Building the project...
echo This may take a few minutes.
echo.

:: Navigate to the rust_simulation directory and build
cd rust_simulation
cargo build --release

if %errorlevel% neq 0 (
    echo Build failed. Please check for errors.
    pause
    exit /b 1
)

echo Build successful. Packaging the application...
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

echo.
echo ======================================================
echo  Application packaged successfully!
echo  The distributable is in the 'dist\windows' directory.
echo  Launching the application now...
echo ======================================================
echo.

:: Run the application
cd "dist\windows"
start rust_simulation.exe

endlocal
