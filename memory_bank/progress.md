# Progress

## 2025-11-05 – v0.2.0 branch, IMAP/SMTP sadeleştirme tamam
- Yeni branch: `v0.2.0` oluşturuldu ve upstream’e pushlandı.
- Kaldırılanlar: `async-smtp` bağımlılığı ve ilgili test endpoint’i; Stalwart API entegrasyonu ve UI öğeleri; JMAP proxy rotaları.
- SMTP test akışı: `/test/smtp/:account_id` yeniden route’a eklendi ve tüm patikalarda JSON dönecek şekilde sabitlendi (UI’de JSON parse hataları giderildi).
- SMTP (lettre): `ClientId::new` kullanımı derleniyor ancak deprecate uyarısı veriyor; ileride `ClientId::Domain` ile temizlenecek (fonksiyonellik etkilenmiyor).
- Sunucu: `cargo run` başarılı; yalnızca uyarılar mevcut. Test UI (static/app.html) sade ve işlevsel, tam mail arayüzü Faz 2’ye ertelendi.

Öne çıkan kararlar
- Sadece IMAP/SMTP (app password) – OAuth2, Stalwart ve JMAP kaldırıldı.
- Backend hataları daima yapılandırılmış JSON dönecek (front-end dayanıklılığı için).

Kısa vadeli odak (Faz 1)
- SMTP sonrası IMAP APPEND ile “Sent” klasörüne kayıt ve UID takibi.
- IMAP delta senkron: folder başına UIDVALIDITY + last_uid persist ve reset stratejisi.
- Bayraklar (\\Seen/\\Flagged/\\Deleted) iki yönlü senkron; Trash/Sent/Junk rol eşlemesi.
- Dayanıklılık: timeout, exponential backoff, idempotency.

---

# Progress Log

## 2024-01-XX - Milestone 1.2 Complete ✅
- ✅ Multi-account management system
- ✅ Account CRUD operations (POST/GET/DELETE)
- ✅ Provider presets (Gmail, Outlook, Yahoo, iCloud, Custom)
- ✅ SQLite database (accounts table)
- ✅ Direct IMAP client (async-imap)
- ✅ Message body fetch with MIME parsing
- ✅ IMAP IDLE watchers (real-time notifications)
- ✅ Web UI (accounts list, message list, message viewer)
- ✅ **Architecture decision: Removed Stalwart dependency**

## Test Results
- ✅ Gmail account connected: cenkorfa1@gmail.com
- ✅ IMAP connection: 1.5s response time
- ✅ Folder discovery: 11 folders, 311 messages in INBOX
- ✅ Message fetch: Headers + body (HTML/Plain text)
- ✅ IDLE watcher: Active and monitoring
- ✅ Web UI: http://localhost:3030/app.html

## Architecture Evolution
### Initial Plan (Abandoned)
- Stalwart as local mail server
- JMAP for frontend
- Mailora as sync-only service

### Current Architecture (Adopted)
- Direct IMAP/SMTP connections
- REST API
- SQLite as primary storage
- Simpler, more maintainable

### Rationale
- Stalwart unnecessary for our use case
- Direct connections more efficient
- Less complexity, easier debugging
- Native IMAP features (IDLE) work perfectly

## Next: Milestone 1.3
- [ ] Message sync to SQLite (messages table schema)
- [ ] SMTP send service (lettre integration)
- [ ] Attachment storage
- [ ] Thread grouping
- [ ] Full-text search (SQLite FTS5)
- [ ] Unified inbox aggregation

## 2025-11-06
- Implemented background finalize with retry (10s x 6) and Gmail-specific raw search. Gmail UID may still be pending within first minute; documented in KNOWN_ISSUES.md. Proceeding to next milestone.

## 2025-11-10 – v0.2.1 unified inbox & minimal istemci
- Unified folder-based inbox endpoint `/unified/inbox` (folder param + unread filter).
- Background scheduler + initial startup sync (non-Gmail) eklendi.
- Body cache TTL + kapasite GC tamamlandı (tablo: `message_bodies`).
- Append policy (auto/never/force) UI testleri ve API güncelleme endpoint’i.
- Basit 3-panelli web istemci (`static/app.html`): hesap ekle, klasör seç, unified toggle, mesaj listesi, önizleme, compose & gönder.
- Flags update route ek insert fallback ile tutarlılık sağlıyor.
- Gmail senaryoları ve kalıcı finalize kuyrukları ertelendi.

### Kalanlar (özet)
- Attachments işleme & indirme
- Unified arama (subject/from/to/date, unread)
- Kalıcı sent finalize & outbox kuyruğu
- Hata zarfı standardizasyonu
- Metrics sayaç doğruluğu
- Index optimizasyonları
- Scheduler backoff/jitter ve IDLE yeniden bağlanma
