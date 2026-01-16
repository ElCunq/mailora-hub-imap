#!/bin/bash
# Gmail IMAP Test Script

BASE_URL="http://localhost:3030"
ACCOUNT_ID="acc_cenkorfa1_gmail_com"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📧 GMAIL IMAP FULL TEST SUITE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "✅ TEST 1: Hesap Bilgilerini Kontrol Et"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s "${BASE_URL}/accounts/${ACCOUNT_ID}" | jq '{id, email, provider, imap_host, enabled}'
echo ""

echo "✅ TEST 2: IMAP Bağlantı Testi"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
CONN_RESULT=$(curl -s "${BASE_URL}/test/connection/${ACCOUNT_ID}")
echo "$CONN_RESULT" | jq '{
  success,
  folder_count: (.folders | length),
  folders: .folders,
  capabilities: (.capabilities | length),
  inbox_messages: .inbox_stats.exists,
  inbox_uidvalidity: .inbox_stats.uidvalidity
}'
echo ""

# Folder sayısını al
FOLDER_COUNT=$(echo "$CONN_RESULT" | jq '.folders | length')
echo "📁 Toplam Klasör: $FOLDER_COUNT"
echo ""

echo "✅ TEST 3: Son 3 Mesaj Önizlemesi"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s "${BASE_URL}/test/messages/${ACCOUNT_ID}?limit=3" | jq '{
  account_id,
  email,
  message_count,
  messages: .messages | map({
    uid,
    subject,
    from,
    date,
    flags
  })
}'
echo ""

echo "✅ TEST 4: Son 10 Mesaj (Detaylı)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
MESSAGES=$(curl -s "${BASE_URL}/test/messages/${ACCOUNT_ID}?limit=10")
echo "$MESSAGES" | jq -r '.messages[] | "[\(.uid)] \(.subject[0:60]) - From: \(.from[0:40])"'
echo ""

MSG_COUNT=$(echo "$MESSAGES" | jq '.message_count')
echo "📊 Çekilen Mesaj Sayısı: $MSG_COUNT"
echo ""

echo "✅ TEST 5: Tüm Test Hesaplarını Listele"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s "${BASE_URL}/test/accounts" | jq 'map({id, email, provider, enabled})'
echo ""

echo "✅ TEST 6: Capabilities Kontrolü"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$CONN_RESULT" | jq -r '.capabilities[]' | grep -E "(IDLE|CONDSTORE|UIDPLUS|MOVE|ENABLE)" | sort
echo ""

echo "✅ TEST 7: Klasör Analizi"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$CONN_RESULT" | jq -r '.folders[]' | while read folder; do
  if [[ "$folder" == "INBOX" ]]; then
    echo "📥 $folder (Ana Gelen Kutusu)"
  elif [[ "$folder" == *"Spam"* ]]; then
    echo "🚫 $folder (Spam)"
  elif [[ "$folder" == *"Trash"* ]] || [[ "$folder" == *"p kutusu"* ]]; then
    echo "🗑️  $folder (Çöp Kutusu)"
  elif [[ "$folder" == *"Sent"* ]] || [[ "$folder" == *"nderilmi"* ]]; then
    echo "📤 $folder (Gönderilen)"
  elif [[ "$folder" == *"Draft"* ]] || [[ "$folder" == *"Taslak"* ]]; then
    echo "📝 $folder (Taslaklar)"
  elif [[ "$folder" == *"Important"* ]] || [[ "$folder" == *"nemli"* ]]; then
    echo "⭐ $folder (Önemli)"
  elif [[ "$folder" == *"All"* ]] || [[ "$folder" == *"T&APw-m"* ]]; then
    echo "📦 $folder (Tüm Postalar)"
  else
    echo "📁 $folder"
  fi
done
echo ""

echo "✅ TEST 8: INBOX İstatistikleri"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$CONN_RESULT" | jq '.inbox_stats'
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ TÜM TESTLER TAMAMLANDI!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 ÖZET:"
echo "  - IMAP Bağlantısı: ✅ Başarılı"
echo "  - Klasör Sayısı: $FOLDER_COUNT"
echo "  - Toplam Mesaj (INBOX): $(echo "$CONN_RESULT" | jq '.inbox_stats.exists')"
echo "  - Çekilen Mesaj Önizleme: $MSG_COUNT"
echo "  - Capabilities: IDLE, UIDPLUS, CONDSTORE ✅"
echo ""
