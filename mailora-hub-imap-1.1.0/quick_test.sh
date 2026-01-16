#!/bin/bash

BASE_URL="http://localhost:3030"

echo "===================================="
echo "✅ Test 1: Provider Listesi"
echo "===================================="
curl -s "${BASE_URL}/providers" | jq '.[].name'
echo ""

echo "===================================="
echo "✅ Test 2: Gmail Hesabı Ekle"
echo "===================================="
RESULT=$(curl -s -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@gmail.com",
    "password": "test_password_123",
    "provider": "gmail",
    "display_name": "Test Gmail Account"
  }')
echo "$RESULT" | jq
GMAIL_ID=$(echo "$RESULT" | jq -r '.id')
echo "Created account ID: $GMAIL_ID"
echo ""

echo "===================================="
echo "✅ Test 3: Outlook Hesabı Ekle"
echo "===================================="
RESULT=$(curl -s -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@outlook.com",
    "password": "outlook_pass",
    "provider": "outlook",
    "display_name": "Test Outlook"
  }')
echo "$RESULT" | jq
OUTLOOK_ID=$(echo "$RESULT" | jq -r '.id')
echo ""

echo "===================================="
echo "✅ Test 4: Custom Provider Hesap"
echo "===================================="
curl -s -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "me@mydomain.com",
    "password": "mypass",
    "provider": "custom",
    "display_name": "Custom Mail Server",
    "imap_host": "mail.mydomain.com",
    "imap_port": 993,
    "smtp_host": "smtp.mydomain.com",
    "smtp_port": 587
  }' | jq
echo ""

echo "===================================="
echo "✅ Test 5: Tüm Hesapları Listele"
echo "===================================="
curl -s "${BASE_URL}/accounts" | jq '.[].email'
echo ""

echo "===================================="
echo "✅ Test 6: Tek Hesap Detayı"
echo "===================================="
if [ ! -z "$GMAIL_ID" ]; then
  curl -s "${BASE_URL}/accounts/${GMAIL_ID}" | jq '{id, email, provider, imap_host, smtp_host, enabled}'
fi
echo ""

echo "===================================="
echo "✅ Test 7: Database Kontrolü"
echo "===================================="
sqlite3 mailora_imap.db "SELECT id, email, provider, enabled FROM accounts;" 2>/dev/null | head -5
echo ""

echo "===================================="
echo "✅ Test 8: Hesap Silme"
echo "===================================="
if [ ! -z "$OUTLOOK_ID" ]; then
  echo "Siliniyor: $OUTLOOK_ID"
  curl -s -X DELETE "${BASE_URL}/accounts/${OUTLOOK_ID}" | jq
  echo ""
  echo "Güncel liste:"
  curl -s "${BASE_URL}/accounts" | jq '.[].email'
fi
echo ""

echo "===================================="
echo "✅ TESTLER TAMAMLANDI!"
echo "===================================="
