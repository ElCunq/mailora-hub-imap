#!/bin/bash
# Account Management API Test Script

BASE_URL="http://localhost:3030"

echo "=========================================="
echo "1. Provider Listesini Getir"
echo "=========================================="
curl -X GET "${BASE_URL}/providers" -H "Content-Type: application/json" | jq .
echo -e "\n"

echo "=========================================="
echo "2. Gmail Hesabı Ekle (Mock Test)"
echo "=========================================="
curl -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@gmail.com",
    "password": "test_password_123",
    "provider": "gmail",
    "display_name": "Test Gmail Account"
  }' | jq .
echo -e "\n"

echo "=========================================="
echo "3. Custom Provider ile Hesap Ekle"
echo "=========================================="
curl -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "test_password_456",
    "provider": "custom",
    "display_name": "Custom Test Account",
    "imap_host": "mail.example.com",
    "imap_port": 993,
    "smtp_host": "smtp.example.com",
    "smtp_port": 587
  }' | jq .
echo -e "\n"

echo "=========================================="
echo "4. Tüm Hesapları Listele"
echo "=========================================="
curl -X GET "${BASE_URL}/accounts" -H "Content-Type: application/json" | jq .
echo -e "\n"

echo "=========================================="
echo "5. Outlook Hesabı Ekle"
echo "=========================================="
curl -X POST "${BASE_URL}/accounts" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@outlook.com",
    "password": "outlook_password",
    "provider": "outlook",
    "display_name": "Test Outlook"
  }' | jq .
echo -e "\n"

echo "=========================================="
echo "6. Tekrar Hesapları Listele (3 hesap olmalı)"
echo "=========================================="
curl -X GET "${BASE_URL}/accounts" -H "Content-Type: application/json" | jq .
echo -e "\n"

echo "=========================================="
echo "7. İlk Hesabı Sil (ID'yi önceki listeden al)"
echo "=========================================="
echo "Lütfen silmek istediğiniz account ID'sini girin:"
read -p "Account ID: " ACCOUNT_ID
if [ ! -z "$ACCOUNT_ID" ]; then
  curl -X DELETE "${BASE_URL}/accounts/${ACCOUNT_ID}" -H "Content-Type: application/json" | jq .
  echo -e "\n"
  
  echo "=========================================="
  echo "8. Son Durumu Kontrol Et"
  echo "=========================================="
  curl -X GET "${BASE_URL}/accounts" -H "Content-Type: application/json" | jq .
fi

echo -e "\n✅ Test tamamlandı!"
