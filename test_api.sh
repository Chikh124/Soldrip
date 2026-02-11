#!/bin/bash

# 🧪 Тестовий скрипт для перевірки soldrip.io API

echo "================================================"
echo "    SOLdrip API Test Script"
echo "================================================"
echo ""

# Змініть це на реальні значення після отримання інформації
API_BASE_URL="https://soldrip.io/api"
TEST_WALLET="TEST_WALLET_ADDRESS_HERE"

echo "🔧 Configuration:"
echo "   Base URL: $API_BASE_URL"
echo "   Test Wallet: $TEST_WALLET"
echo ""

# Кольори для виводу
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Функція для тестування endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 Testing: $description"
    echo "   Method: $method"
    echo "   Endpoint: $endpoint"

    if [ ! -z "$data" ]; then
        echo "   Data: $data"
    fi

    echo ""

    if [ "$method" == "GET" ]; then
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -X GET "$endpoint")
    else
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -X POST "$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi

    # Розділяємо body та status code
    body=$(echo "$response" | sed -e 's/HTTP_STATUS\:.*//g')
    status=$(echo "$response" | tr -d '\n' | sed -e 's/.*HTTP_STATUS://')

    if [ "$status" -ge 200 ] && [ "$status" -lt 300 ]; then
        echo -e "${GREEN}✅ Success (HTTP $status)${NC}"
    else
        echo -e "${RED}❌ Failed (HTTP $status)${NC}"
    fi

    echo "Response:"
    echo "$body" | jq . 2>/dev/null || echo "$body"
    echo ""
}

# Тест 1: Підключення гаманця
test_endpoint \
    "POST" \
    "$API_BASE_URL/wallet/connect" \
    "{\"wallet_address\":\"$TEST_WALLET\"}" \
    "Connect Wallet"

# Тест 2: Перевірка балансу
test_endpoint \
    "GET" \
    "$API_BASE_URL/wallet/balance/$TEST_WALLET" \
    "" \
    "Get Balance"

# Тест 3: Claim
test_endpoint \
    "POST" \
    "$API_BASE_URL/wallet/claim" \
    "{\"wallet_address\":\"$TEST_WALLET\"}" \
    "Claim SOL"

# Тест 4: Статистика (якщо є)
test_endpoint \
    "GET" \
    "$API_BASE_URL/stats" \
    "" \
    "Get Stats"

echo "================================================"
echo "    Tests Complete!"
echo "================================================"
echo ""
echo "💡 Tips:"
echo "   - Якщо всі тести fail - перевірте API_BASE_URL"
echo "   - Якщо 401/403 - потрібна авторизація"
echo "   - Якщо 404 - неправильний endpoint"
echo "   - Якщо 429 - rate limiting"
echo ""
