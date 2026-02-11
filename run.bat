@echo off
echo ================================================
echo    SOLdrip Automation Tool - Quick Start
echo ================================================
echo.

REM Перевірка наявності Rust
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Rust is not installed!
    echo.
    echo Please install Rust from: https://rustup.rs/
    echo.
    echo After installation, restart this script.
    pause
    exit /b 1
)

echo [OK] Rust detected:
cargo --version
echo.

echo Building project in release mode...
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo ================================================
echo    Starting SOLdrip Automation Tool...
echo ================================================
echo.

cargo run --release

pause
