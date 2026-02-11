@echo off
REM 🧪 Тестовий скрипт для перевірки soldrip.io API (Windows)

echo ================================================
echo     SOLdrip API Test Script
echo ================================================
echo.

REM Змініть це на реальні значення після отримання інформації
set API_BASE_URL=https://soldrip.io/api
set TEST_WALLET=TEST_WALLET_ADDRESS_HERE

echo Configuration:
echo    Base URL: %API_BASE_URL%
echo    Test Wallet: %TEST_WALLET%
echo.

REM Перевірка наявності curl
where curl >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] curl is not installed!
    echo Please install curl or use WSL
    pause
    exit /b 1
)

echo ================================================
echo Test 1: Connect Wallet
echo ================================================
curl -X POST %API_BASE_URL%/wallet/connect ^
    -H "Content-Type: application/json" ^
    -d "{\"wallet_address\":\"%TEST_WALLET%\"}"
echo.
echo.

echo ================================================
echo Test 2: Get Balance
echo ================================================
curl -X GET %API_BASE_URL%/wallet/balance/%TEST_WALLET%
echo.
echo.

echo ================================================
echo Test 3: Claim SOL
echo ================================================
curl -X POST %API_BASE_URL%/wallet/claim ^
    -H "Content-Type: application/json" ^
    -d "{\"wallet_address\":\"%TEST_WALLET%\"}"
echo.
echo.

echo ================================================
echo     Tests Complete!
echo ================================================
echo.
echo Tips:
echo    - If all tests fail - check API_BASE_URL
echo    - If 401/403 - need authorization
echo    - If 404 - wrong endpoint
echo    - If 429 - rate limiting
echo.

pause
