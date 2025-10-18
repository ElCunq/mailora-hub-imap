# Progress

What works:
- /diff multi-folder aggregation; minimal emission when meta missing.
- /body refetch flow with SEARCH and multiple UID query forms; detailed logs.
- **Multi-Account Management:** Add/list/get/delete accounts via API
- **Provider Presets:** Gmail, Outlook, Yahoo, iCloud default configs
- **Accounts Table:** SQLite schema with provider, credentials, sync settings

What's left:
- BODYSTRUCTURE-driven precise part selection.
- Better error semantics to UI on fetch-empty when UID present.
- **External IMAP sync:** Per-account IMAP connection & message fetch
- **IDLE watchers:** Background tasks for each account
- **Credential security:** Upgrade from base64 to OS keychain

Status:
- Multi-account management API ready for testing
- Provider presets configured (Gmail, Outlook, Yahoo, iCloud, Custom)
- Accounts table created with proper schema

Known issues:
- Occasionally UID exists but FETCH returns empty; mitigated by retries/NOOP.
- SQLite type mismatches in account_service (boolean/integer) - IN PROGRESS
- Credential storage currently base64 (needs OS keychain integration)

Evolution:
- Added folder to ChangeItem; expanded logging; created memory_bank for continuity.
- **NEW:** Multi-provider account management system
- **NEW:** Account CRUD endpoints (/accounts, /providers)
- **NEW:** Email provider abstraction layer

---

Meta
- Güncelleme: 2025-10-10
- Sürüm: 0.2
- Sahip: TODO

Yakın Vade Yapılacaklar
- BODYSTRUCTURE ile parça-seçimli fetch akışını uygulama ve test etme.
- FETCH-empty (UID mevcut ama gövde yok) durumları için UI’ye yapılandırılmış hata kodları sağlama.
- Gmail, O365, Dovecot, Fastmail üzerinde uçtan uca duman testleri.

Kabul Kriterleri
- BODYSTRUCTURE odaklı seçim:
  - multipart/alternative içinde text/plain öncelikli, yoksa text/html fallback.
  - multipart/mixed ve gömülü message/rfc822 için ek parça/ek sayımı doğru.
  - Tek sefer BODYSTRUCTURE al, sonra yalnız gerekli BODY[<section>] parçalarını getir; gereksiz byte indirimi < 30%.
  - 50 rastgele mail üzerinde decode (charset, transfer-encoding) hatası: 0.
- FETCH-empty hata semantiği:
  - IMAP_EMPTY_FETCH, IMAP_UID_STALE, IMAP_PERM gibi ayrık kodlar ve kullanıcıya eylem önerisi.
  - Yeniden deneme politikası: 3 deneme, NOOP/IDLE ile tazeleme; metriklere işlenmiş.

Tanılama ve Gözlemlenebilirlik
- Oturum başında CAPABILITY, PERMANENTFLAGS, UIDVALIDITY, HIGHESTMODSEQ logla.
- BODYSTRUCTURE ağacını tek satır özetle (tip, alt-tip, parça sayısı, en büyük ek boyutu).
- FETCH/SEARCH istek/yanıt sayaçları ve toplam indirilen byte metriği.
- FETCH-empty vakalarında: kullanılan UID listesi, SEARCH sonucunun ham çıktısı ve server tag’ı kaydı.

Test Matrisi
- Sunucular: Gmail, O365/Outlook, Fastmail, Dovecot.
- Klasörler: INBOX, All Mail/Archive, rastgele etiket.
- Durumlar: büyük ek (>10MB), yalnız HTML, yalnız Plain, nested message/rfc822, bozuk MIME sınırları.
- Sorgular: tek UID, aralıklı UID seti, karışık UID ve SEQ, ESEARCH destekli/desteksiz.

Performans Bütçesi
- 100 ileti için toplam istek sayısı ≤ 12 (SEARCH + 2 FETCH dalgası maksimumu).
- Ortalama indirilen byte/ileti ≤ 80KB (ekler hariç gövde + başlık).
- Zaman bütçesi: 100 ileti ≤ 3.5 sn (Gmail) ve ≤ 4.5 sn (IMAP genel) 50ms RTT’de.

Karar Günlüğü
- UID tabanlı FETCH tercih edildi; sıra numarası yalnız kısa ömürlü oturumlarda fallback.
- BODYSTRUCTURE ön-okuma zorunlu; tam RFC822 indirme yalnız hatada veya kullanıcı isteğinde.
- Yeniden deneme: sabit 250ms artan bekleme ile 3 deneme; NOOP sonrası tek deneme daha.

Açık Sorular ve Riskler
- BODYSTRUCTURE cache’i (UID→ağaç) için TTL ve invalidation stratejisi.
- Gmail X-GM-EXT parametreleri ile optimize arama; taşınabilirlik etkisi.
- Paralellik: klasör başına eşzamanlı FETCH sınırı (öneri: 3-5).

Playbook Ekleri
- playbooks/body_fetch.md içine "FETCH-empty triage" ve "BODYSTRUCTURE okunması" bölümleri eklenecek.
- facts/imap.md içine sunucu farklılıkları (Gmail/Outlook/Dovecot) tablosu eklenecek.

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
