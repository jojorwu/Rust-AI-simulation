@echo off
setlocal

:: This script is a simple wrapper to launch the main PowerShell setup script.
:: It ensures that users can simply double-click this file to start the process.

:: Get the directory where this script is located
set "SCRIPT_DIR=%~dp0"

:: Launch the PowerShell script
:: -ExecutionPolicy Bypass: Ensures the script runs even if the system policy is restrictive.
:: -NoProfile: Speeds up the start time by not loading user profiles.
:: -File: Specifies the script to run.
echo Launching the PowerShell setup script...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%setup-windows.ps1"

echo.
echo Script finished. Press any key to exit.
pause >nul
endlocal
