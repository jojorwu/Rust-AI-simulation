@echo off
echo Building the project in release mode...
cargo build --release

REM Check if the build was successful
if %errorlevel% neq 0 (
    echo Build failed. Please check for errors.
    pause
    exit /b %errorlevel%
)

echo Creating distribution directory...
if not exist "dist" mkdir "dist"

echo Copying executable to dist directory...
copy "target\release\rust_simulation.exe" "dist\"

echo Copying data files to dist directory...
xcopy "data" "dist\data\" /E /I /Y

echo.
echo ======================================================
echo  Build successful!
echo  A distributable version is available in the 'dist' folder.
echo  You can now run the application from there.
echo ======================================================
echo.

pause
