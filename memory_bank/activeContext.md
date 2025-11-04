# Active Context

Current focus:
- **Milestone 1.1 (Multi-Account Management) ✅ TAMAMLANDI**
- **Gmail IMAP Testi ✅ BAŞARILI** - 311 mesaj, 11 klasör
- **Sonraki Adım:** Milestone 1.2 - IDLE Watchers & Real-time Sync

Recent changes (3 Kasım 2025):
- ✅ IMAP test servisi oluşturuldu (`src/services/imap_test_service.rs`)
- ✅ Test endpoints eklendi (`src/routes/test.rs`)
  - `GET /test/connection/:account_id` - IMAP bağlantı testi
  - `GET /test/messages/:account_id?limit=N` - Mesaj önizleme
  - `GET /test/accounts` - Test hesap listesi
- ✅ Gerçek Gmail hesabı test edildi (cenkorfa1@gmail.com)
  - 311 mesaj INBOX'ta
  - 11 klasör başarıyla listelendi
  - IDLE, UIDPLUS, CONDSTORE capabilities doğrulandı
Test Sonuçları:
- ✅ IMAP bağlantı: 1.5 saniye
- ✅ Folder listesi: 11 klasör (INBOX, Spam, Sent, etc.)
- ✅ Message fetch: 10 mesaj/1.5 saniye
- ✅ Capabilities: IDLE, CONDSTORE, UIDPLUS ✓
- ✅ `test_gmail_imap.sh` - Kapsamlı test scripti
- ✅ `GMAIL_TEST_RESULTS.md` - Detaylı test raporu

Test Sonuçları:
- ✅ IMAP bağlantı: 1.5 saniye
- ✅ Folder listesi: 11 klasör (INBOX, Spam, Sent, etc.)
- ✅ Message fetch: 10 mesaj/1.5 saniye
- ✅ Capabilities: IDLE, CONDSTORE, UIDPLUS ✓

## Next Steps (Milestone 1.3)
1. ✅ ~~Test account CRUD endpoints~~
2. ✅ ~~Test with real Gmail account~~
3. ✅ ~~Message body fetch~~
4. ✅ ~~IDLE watchers~~
5. **TODO: Message sync to SQLite** (messages table)
6. **TODO: SMTP send service** (lettre integration)
7. **TODO: Attachment handling**
8. **TODO: Thread grouping**
9. **TODO: Full-text search** (SQLite FTS5)
10. **TODO: Unified inbox UI**

Decisions:
- Provider enum: gmail, outlook, yahoo, icloud, custom
- Account ID format: `acc_{email_sanitized}` (ör: acc_user_gmail_com)
- Credentials: Base64 encoded "email:password" (geçici, production'da OS keychain)
- Sync frequency default: 300 saniye (5 dakika)
- IMAP/SMTP defaults provider'dan otomatik

Milestones:
- ✅ M0: IDLE + meta listing + basic FETCH (completed)
- ✅ M1: IMAP IDLE watcher + SMTP send + event log + unified endpoints (COMPLETED)
- ✅ M1.1: Multi-Account Management (TAMAMLANDI - test bekliyor)
- ⏳ M1.2: External IMAP sync per account + IDLE watchers
- ⏳ M2: BODYSTRUCTURE selection + lazy fetch
- ⏳ M3: Personal mode Stalwart embedding
- ⏳ M4: Enterprise RBAC visibility
